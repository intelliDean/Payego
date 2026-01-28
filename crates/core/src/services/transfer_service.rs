use crate::client::PaystackClient;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
use diesel::prelude::*;
use payego_primitives::models::dtos::providers::paystack::{
    PaystackTransferData, PaystackTransferResponse,
};
pub use payego_primitives::{
    config::security_config::Claims,
    error::ApiError,
    models::{
        app_state::AppState,
        dtos::wallet_dto::{TransferRequest, TransferResponse, WalletTransferRequest},
        enum_types::{CurrencyCode, PaymentProvider, PaymentState, TransactionIntent},
        transaction::{NewTransaction, Transaction},
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
    schema::{transactions, wallet_ledger, wallets},
};
use reqwest::{Client, Url};
use secrecy::ExposeSecret;
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub struct TransferService;

impl TransferService {
    pub async fn transfer_internal(
        state: &Arc<AppState>,
        sender_id: Uuid,
        // recipient_id: Uuid,
        req: WalletTransferRequest,
    ) -> Result<Uuid, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let amount_cents = (req.amount * 100.0).round() as i64;
        if amount_cents <= 0 {
            return Err(ApiError::Internal("Amount must be positive".into()));
        }

        conn.transaction::<_, ApiError, _>(|conn| {
            // ── 1. Idempotency
            if let Some(existing) = TransactionRepository::find_by_idempotency_key(
                conn,
                sender_id,
                &req.idempotency_key,
            )? {
                if existing.txn_state == PaymentState::Completed {
                    info!(
                        transaction_id = %existing.id,
                        idempotency_key = %req.idempotency_key,
                        "Internal transfer already completed (idempotency check)"
                    );
                    return Ok(existing.id);
                }
            }

            // ── 2. Lock wallets
            let sender_wallet = WalletRepository::find_by_user_and_currency_with_lock(
                conn,
                sender_id,
                req.currency,
            )?;
            let recipient_wallet =
                WalletRepository::create_if_not_exists(conn, req.recipient, req.currency)?;

            if sender_wallet.balance < amount_cents {
                warn!(
                    user_id = %sender_id,
                    available_balance = sender_wallet.balance,
                    requested_amount = amount_cents,
                    currency = %req.currency,
                    "Insufficient balance for internal transfer"
                );
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            // ── 3. Sender transaction (debit)
            let sender_tx = TransactionRepository::create(
                conn,
                NewTransaction {
                    user_id: sender_id,
                    counterparty_id: Some(req.recipient),
                    intent: TransactionIntent::Transfer,
                    amount: amount_cents,
                    currency: req.currency,
                    txn_state: PaymentState::Completed,
                    provider: Some(PaymentProvider::Internal),
                    provider_reference: None,
                    idempotency_key: &req.idempotency_key,
                    reference: req.reference,
                    description: req.description.as_deref(),
                    metadata: json!({
                        "direction": "debit",
                        "counterparty": req.recipient,
                    }),
                },
            )?;

            // ── 4. Recipient transaction (credit)
            let recipient_tx = TransactionRepository::create(
                conn,
                NewTransaction {
                    user_id: req.recipient,
                    counterparty_id: Some(sender_id),
                    intent: TransactionIntent::Transfer,
                    amount: amount_cents,
                    currency: req.currency,
                    txn_state: PaymentState::Completed,
                    provider: Some(PaymentProvider::Internal),
                    provider_reference: None,
                    idempotency_key: &req.idempotency_key,
                    reference: Uuid::new_v4(),
                    description: Some("Internal transfer received"),
                    metadata: json!({
                        "direction": "credit",
                        "counterparty": sender_id,
                        "original_reference": req.reference
                    }),
                },
            )?;

            // ── 5. Ledger entries
            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: sender_wallet.id,
                    transaction_id: sender_tx.id,
                    amount: -amount_cents,
                },
            )?;
            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: recipient_wallet.id,
                    transaction_id: recipient_tx.id,
                    amount: amount_cents,
                },
            )?;

            // ── 6. Update balances
            WalletRepository::debit(conn, sender_wallet.id, amount_cents)?;
            WalletRepository::credit(conn, recipient_wallet.id, amount_cents)?;

            info!(
                transaction_id = %sender_tx.id,
                sender_id = %sender_id,
                recipient_id = %req.recipient,
                amount = amount_cents,
                currency = %req.currency,
                "Internal transfer completed successfully"
            );

            Ok(sender_tx.id)
        })
    }

    pub async fn transfer_external(
        state: &Arc<AppState>,
        user_id: Uuid,
        req: TransferRequest,
    ) -> Result<TransferResponse, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let currency: CurrencyCode = req
            .currency
            .parse()
            .map_err(|_| ApiError::Internal("Unsupported currency".into()))?;

        let amount_minor = (req.amount * 100.0).round() as i64;
        if amount_minor <= 0 {
            return Err(ApiError::Internal("Amount must be positive".into()));
        }

        let tx_id = conn.transaction::<_, ApiError, _>(|conn| {
            // ── 1. Idempotency
            if let Some(existing) =
                TransactionRepository::find_by_idempotency_key(conn, user_id, &req.idempotency_key)?
            {
                info!(
                    transaction_id = %existing.id,
                    idempotency_key = %req.idempotency_key,
                    "External transfer already initiated (idempotency check)"
                );
                return Ok(existing.id);
            }

            // ── 2. Lock wallet
            let wallet =
                WalletRepository::find_by_user_and_currency_with_lock(conn, user_id, currency)?;

            if wallet.balance < amount_minor {
                warn!(
                    user_id = %user_id,
                    available_balance = wallet.balance,
                    requested_amount = amount_minor,
                    currency = %currency,
                    "Insufficient balance for external transfer"
                );
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            // ── 3. Create pending payout transaction
            let tx = TransactionRepository::create(
                conn,
                NewTransaction {
                    user_id,
                    counterparty_id: None,
                    intent: TransactionIntent::Payout,
                    amount: amount_minor,
                    currency,
                    txn_state: PaymentState::Pending,
                    provider: Some(PaymentProvider::Paystack),
                    provider_reference: None,
                    idempotency_key: &req.idempotency_key,
                    reference: req.reference,
                    description: Some("External bank transfer"),
                    metadata: json!({
                        "bank_code": req.bank_code,
                        "account_number": req.account_number,
                        "account_name": req.account_name,
                    }),
                },
            )?;

            // ── 4. Ledger reservation (funds held)
            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: tx.id,
                    amount: -amount_minor,
                },
            )?;

            // ── 5. Reduce available balance
            WalletRepository::debit(conn, wallet.id, amount_minor)?;

            Ok(tx.id)
        })?;

        // ── 6. Call Paystack OUTSIDE DB transaction
        let provider_data = Self::initiate_paystack_transfer(state, &req).await?;

        // ── 7. Attach provider reference and update state if success
        TransactionRepository::update_status_and_provider_ref(
            &mut conn,
            tx_id,
            if provider_data.status.as_deref() == Some("success") {
                PaymentState::Completed
            } else {
                PaymentState::Pending
            },
            Some(provider_data.transfer_code.to_string()),
        )?;

        info!(
            transaction_id = %tx_id,
            user_id = %user_id,
            amount = amount_minor,
            currency = %currency,
            provider_status = ?provider_data.status,
            "External transfer initiated successfully"
        );

        Ok(TransferResponse {
            transaction_id: tx_id,
        })
    }

    async fn initiate_paystack_transfer(
        state: &AppState,
        req: &TransferRequest,
    ) -> Result<PaystackTransferData, ApiError> {
        let client = Client::new();
        let key = state
            .config
            .paystack_details
            .paystack_secret_key
            .expose_secret();

        let paystack_client = PaystackClient::new(
            state.http_client.clone(),
            &state.config.paystack_details.paystack_api_url,
            state.config.paystack_details.paystack_secret_key.clone(),
        )?;

        let name = req.account_name.clone().unwrap_or("Recipient".into());

        let payload = PaystackClient::create_recipient(
            &name,
            &req.account_number,
            &req.bank_code,
            CurrencyCode::NGN,
        );

        let recipient_code = paystack_client
            .create_transfer_recipient(payload)
            .await
            .map_err(|_| ApiError::Payment("Unable to create transfer recipient".into()))?;

        //todo: turn this into client
        let base = Url::parse(&state.config.paystack_details.paystack_api_url)
            .map_err(|_| ApiError::Internal("Invalid Paystack base URL".into()))?;

        let url = base
            .join("transfer")
            .map_err(|_| ApiError::Internal("Invalid Paystack transfer URL".into()))?;

        let amount_kobo = (req.amount * 100.0).round() as i64;

        if amount_kobo <= 0 {
            return Err(ApiError::Internal("Invalid transfer amount".into()));
        }

        let resp = client
            .post(url)
            .bearer_auth(key)
            .json(&serde_json::json!({
                "source": "balance",
                "amount": amount_kobo,
                "recipient": recipient_code,
                "reference": req.reference.to_string()
            }))
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Paystack transfer request failed");
                ApiError::Payment("Failed to reach Paystack".into())
            })?
            .error_for_status()
            .map_err(|e| {
                tracing::warn!(error = %e, "Paystack transfer rejected");
                ApiError::Payment("Paystack rejected transfer".into())
            })?;

        //todo===== end of client

        let body: PaystackTransferResponse = resp.json().await.map_err(|e| {
            tracing::error!(error = %e, "Invalid Paystack transfer response");
            ApiError::Payment("Invalid Paystack response".into())
        })?;

        if !body.status {
            return Err(ApiError::Payment(body.message));
        }

        body.data
            .ok_or_else(|| ApiError::Payment("Missing transfer data".into()))
    }
}
