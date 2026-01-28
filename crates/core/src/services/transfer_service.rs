pub use crate::app_state::AppState;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
pub use crate::security::Claims;
use crate::services::audit_service::AuditService;
use diesel::prelude::*;
pub use payego_primitives::{
    error::ApiError,
    models::{
        dtos::wallet_dto::{TransferRequest, TransferResponse, WalletTransferRequest},
        enum_types::{CurrencyCode, PaymentProvider, PaymentState, TransactionIntent},
        transaction::NewTransaction,
        wallet_ledger::NewWalletLedger,
    },
};
use serde_json::json;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct TransferService;

impl TransferService {
    pub async fn transfer_internal(
        state: &Arc<AppState>,
        sender_id: Uuid,
        req: WalletTransferRequest,
    ) -> Result<Uuid, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let amount_cents = req.amount;
        if amount_cents <= 0 {
            return Err(ApiError::Internal("Amount must be positive".into()));
        }

        let tx_id = conn.transaction::<_, ApiError, _>(|conn| {
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

            let sender_wallet = WalletRepository::find_by_user_and_currency_with_lock(
                conn,
                sender_id,
                req.currency,
            )?;
            let recipient_wallet =
                WalletRepository::create_if_not_exists(conn, req.recipient, req.currency)?;

            if sender_wallet.balance < amount_cents {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

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

            WalletRepository::debit(conn, sender_wallet.id, amount_cents)?;
            WalletRepository::credit(conn, recipient_wallet.id, amount_cents)?;

            Ok::<Uuid, ApiError>(sender_tx.id)
        })?;

        let _ = AuditService::log_event(
            state,
            Some(sender_id),
            "transfer.internal",
            Some("transaction"),
            Some(&tx_id.to_string()),
            json!({
                "recipient": req.recipient,
                "amount": amount_cents,
                "currency": req.currency,
            }),
            None,
        )
        .await;

        Ok(tx_id)
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

        let amount_minor = req.amount;
        if amount_minor <= 0 {
            return Err(ApiError::Internal("Amount must be positive".into()));
        }

        let tx_id = conn.transaction::<_, ApiError, _>(|conn| {
            if let Some(existing) =
                TransactionRepository::find_by_idempotency_key(conn, user_id, &req.idempotency_key)?
            {
                return Ok(existing.id);
            }

            let wallet =
                WalletRepository::find_by_user_and_currency_with_lock(conn, user_id, currency)?;

            if wallet.balance < amount_minor {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

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

            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: tx.id,
                    amount: -amount_minor,
                },
            )?;

            WalletRepository::debit(conn, wallet.id, amount_minor)?;

            Ok::<Uuid, ApiError>(tx.id)
        })?;

        let name = req.account_name.clone().unwrap_or("Recipient".into());
        let payload = crate::clients::paystack::PaystackClient::create_recipient_payload(
            &name,
            &req.account_number,
            &req.bank_code,
            CurrencyCode::NGN,
        );

        let recipient_code = state
            .paystack
            .create_transfer_recipient(payload)
            .await
            .map_err(|_| ApiError::Payment("Unable to create transfer recipient".into()))?;

        state
            .paystack
            .initiate_transfer(&recipient_code, amount_minor, &req.reference.to_string())
            .await?;

        // Updating status to completed (simplified, usually we'd wait for webhook)
        TransactionRepository::update_status_and_provider_ref(
            &mut conn,
            tx_id,
            PaymentState::Completed,
            Some(recipient_code), // Using recipient_code as provider ref for now
        )?;

        let _ = AuditService::log_event(
            state,
            Some(user_id),
            "transfer.external",
            Some("transaction"),
            Some(&tx_id.to_string()),
            json!({
                "bank_code": req.bank_code,
                "amount": amount_minor,
                "currency": currency,
            }),
            None,
        )
        .await;

        Ok(TransferResponse {
            transaction_id: tx_id,
        })
    }
}
