pub use crate::app_state::AppState;
use crate::repositories::bank_account_repository::BankAccountRepository;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
pub use crate::security::Claims;
use crate::services::audit_service::AuditService;
use diesel::prelude::*;
pub use payego_primitives::{
    error::ApiError,
    models::{
        dtos::wallet_dto::{WithdrawRequest, WithdrawResponse},
        enum_types::{PaymentProvider, PaymentState, TransactionIntent},
        transaction::NewTransaction,
        wallet_ledger::NewWalletLedger,
    },
};
use serde_json::json;
use uuid::Uuid;

pub struct WithdrawalService;

impl WithdrawalService {
    pub async fn withdraw(
        state: &AppState,
        user_id: Uuid,
        bank_account_id: Uuid,
        req: WithdrawRequest,
    ) -> Result<WithdrawResponse, ApiError> {
        let amount_minor = req.amount;
        if amount_minor <= 0 {
            return Err(ApiError::Internal("Invalid amount".into()));
        }

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let bank_account = BankAccountRepository::find_verified_by_id_and_user(
            &mut conn,
            bank_account_id,
            user_id,
        )?;

        let recipient_code = bank_account
            .provider_recipient_id
            .as_deref()
            .ok_or_else(|| {
                ApiError::Payment("Bank account is not linked to a provider recipient".into())
            })?;

        // 1. Transactional DB setup
        let (tx_id, currency) = conn.transaction::<_, ApiError, _>(|conn| {
            // 2. Idempotency Check
            if let Some(existing) =
                TransactionRepository::find_by_idempotency_key(conn, user_id, &req.idempotency_key)?
            {
                if existing.txn_state == PaymentState::Completed {
                    return Ok((existing.id, existing.currency));
                }
                // If it's Pending, we'll fall through and retry the external call
                if existing.txn_state == PaymentState::Pending {
                    return Ok((existing.id, existing.currency));
                }

                return Err(ApiError::Payment(
                    "Transaction already exists with a different state".into(),
                ));
            }

            // 3. Wallet Lock & Balance Check
            let wallet =
                WalletRepository::find_by_user_and_currency_with_lock(conn, user_id, req.currency)?;

            if wallet.balance < amount_minor {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            // 4. Create PENDING Transaction & Debit Wallet
            let tx = TransactionRepository::create(
                conn,
                NewTransaction {
                    user_id,
                    counterparty_id: None,
                    intent: TransactionIntent::Payout,
                    amount: amount_minor,
                    currency: wallet.currency,
                    txn_state: PaymentState::Pending,
                    provider: Some(PaymentProvider::Paystack),
                    provider_reference: None,
                    idempotency_key: &req.idempotency_key,
                    reference: req.reference,
                    description: Some("Wallet withdrawal"),
                    metadata: json!({
                        "bank_code": bank_account.bank_code,
                        "account_number": bank_account.account_number,
                    }),
                },
            )?;

            WalletRepository::debit(conn, wallet.id, amount_minor)?;

            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: tx.id,
                    amount: -amount_minor,
                },
            )?;

            Ok::<(Uuid, payego_primitives::models::enum_types::CurrencyCode), ApiError>((
                tx.id,
                wallet.currency,
            ))
        })?;

        // 5. External transfer (Safe to retry if Pending)
        let paystack_result = state
            .paystack
            .initiate_transfer(recipient_code, amount_minor, &req.reference.to_string())
            .await;

        match paystack_result {
            Ok(_) => {
                // 6. Complete Transaction
                TransactionRepository::update_status_and_provider_ref(
                    &mut conn,
                    tx_id,
                    PaymentState::Completed,
                    Some(recipient_code.to_string()),
                )?;
            }
            Err(e) => {
                tracing::error!(error = %e, transaction_id = %tx_id, "Paystack transfer call failed");
                return Err(e);
            }
        }

        let _ = AuditService::log_event(
            state,
            Some(user_id),
            "withdrawal.initiated",
            Some("transaction"),
            Some(&tx_id.to_string()),
            json!({
                "amount": amount_minor,
                "currency": currency,
                "bank_account_id": bank_account_id,
            }),
            None,
        )
        .await;

        Ok(WithdrawResponse {
            transaction_id: tx_id,
        })
    }
}
