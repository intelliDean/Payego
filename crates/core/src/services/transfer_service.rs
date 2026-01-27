use crate::client::PaystackClient;
use diesel::prelude::*;
use http::StatusCode;
use payego_primitives::models::providers_dto::{PaystackTransferData, PaystackTransferResponse};
use payego_primitives::models::wallet::NewWallet;
pub use payego_primitives::{
    config::security_config::Claims,
    error::ApiError,
    models::{
        app_state::AppState,
        enum_types::{CurrencyCode, PaymentProvider, PaymentState, TransactionIntent},
        transaction::{NewTransaction, Transaction},
        transfer_dto::{TransferRequest, TransferResponse, WalletTransferRequest},
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
    schema::{transactions, wallet_ledger, wallets},
};
use reqwest::{Client, Url};
use secrecy::ExposeSecret;
use serde_json::json;
use std::sync::Arc;
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
            // ── 1. Idempotency (inside TX, enforced logically)
            if let Some(existing) = transactions::table
                .filter(transactions::user_id.eq(sender_id))
                .filter(transactions::idempotency_key.eq(&req.idempotency_key))
                .first::<Transaction>(conn)
                .optional()?
            {
                if existing.txn_state == PaymentState::Completed {
                    return Ok(existing.id);
                }
            }

            // ── 2. Lock wallets
            let sender_wallet = wallets::table
                .filter(wallets::user_id.eq(sender_id))
                .filter(wallets::currency.eq(&req.currency))
                .for_update()
                .first::<Wallet>(conn)
                .map_err(|_| ApiError::Payment("Sender wallet not found".into()))?;

            let recipient_wallet = get_or_create_wallet(conn, req.recipient, &req.currency)?;

            if sender_wallet.balance < amount_cents {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            // ── 3. Sender transaction (debit)
            let sender_tx_id = diesel::insert_into(transactions::table)
                .values(NewTransaction {
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
                })
                .returning(transactions::id)
                .get_result::<Uuid>(conn)?;

            // ── 4. Recipient transaction (credit)
            let recipient_tx_id = diesel::insert_into(transactions::table)
                .values(NewTransaction {
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
                })
                .returning(transactions::id)
                .get_result::<Uuid>(conn)?;

            // ── 5. Ledger entries
            diesel::insert_into(wallet_ledger::table)
                .values(&[
                    NewWalletLedger {
                        wallet_id: sender_wallet.id,
                        transaction_id: sender_tx_id,
                        amount: -amount_cents,
                    },
                    NewWalletLedger {
                        wallet_id: recipient_wallet.id,
                        transaction_id: recipient_tx_id,
                        amount: amount_cents,
                    },
                ])
                .execute(conn)?;

            // ── 6. Update balances
            diesel::update(wallets::table)
                .filter(wallets::id.eq(sender_wallet.id))
                .set(wallets::balance.eq(wallets::balance - amount_cents))
                .execute(conn)?;

            diesel::update(wallets::table)
                .filter(wallets::id.eq(recipient_wallet.id))
                .set(wallets::balance.eq(wallets::balance + amount_cents))
                .execute(conn)?;

            Ok(sender_tx_id)
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
            // ── 1. Idempotency (hard guarantee)
            if let Some(existing) = transactions::table
                .filter(transactions::user_id.eq(user_id))
                .filter(transactions::idempotency_key.eq(&req.idempotency_key))
                .first::<Transaction>(conn)
                .optional()?
            {
                return Ok(existing.id);
            }

            // ── 2. Lock wallet
            let wallet = wallets::table
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(currency))
                .for_update()
                .first::<Wallet>(conn)
                .map_err(|_| ApiError::Payment("Wallet not found".into()))?;

            if wallet.balance < amount_minor {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            // ── 3. Create pending payout transaction
            let tx_id = diesel::insert_into(transactions::table)
                .values(NewTransaction {
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
                })
                .returning(transactions::id)
                .get_result::<Uuid>(conn)?;

            // ── 4. Ledger reservation (funds held)
            diesel::insert_into(wallet_ledger::table)
                .values(NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: tx_id,
                    amount: -amount_minor,
                })
                .execute(conn)?;

            // ── 5. Reduce available balance
            diesel::update(wallets::table)
                .filter(wallets::id.eq(wallet.id))
                .set(wallets::balance.eq(wallets::balance - amount_minor))
                .execute(conn)?;

            Ok(tx_id)
        })?;

        // ── 6. Call Paystack OUTSIDE DB transaction
        let provider_data = Self::initiate_paystack_transfer(state, &req).await?;

        // ── 7. Attach provider reference and update state if success
        diesel::update(transactions::table)
            .filter(transactions::reference.eq(req.reference))
            .set((
                transactions::provider_reference.eq(Some(provider_data.transfer_code.as_str())),
                transactions::txn_state.eq(if provider_data.status.as_deref() == Some("success") {
                    PaymentState::Completed
                } else {
                    PaymentState::Pending
                }),
            ))
            .execute(&mut conn)?;

        Ok(TransferResponse { transaction_id: tx_id })
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

pub fn get_or_create_wallet(
    conn: &mut PgConnection,
    user_id: Uuid,
    currency: &CurrencyCode,
) -> Result<Wallet, ApiError> {
    // use crate::schema::wallets::dsl::*;

    // Try to fetch existing wallet with row lock
    if let Ok(wallet) = wallets::table
        .filter(wallets::user_id.eq(&user_id))
        .filter(wallets::currency.eq(currency))
        .for_update()
        .first::<Wallet>(conn)
    {
        return Ok(wallet);
    }

    // Wallet does not exist → create it
    let new_wallet = NewWallet {
        user_id,
        currency: currency.clone(),
    };

    diesel::insert_into(wallets::table)
        .values(&new_wallet)
        .on_conflict((wallets::user_id, wallets::currency))
        .do_nothing()
        .execute(conn)
        .map_err(ApiError::Database)?;

    // Re-fetch with lock (important)
    wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(currency))
        .for_update()
        .first::<Wallet>(conn)
        .map_err(|_| ApiError::Payment("Failed to create wallet".into()))
}
