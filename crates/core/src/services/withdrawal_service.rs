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

        let wallet = WalletRepository::find_by_user_and_currency_with_lock(
            &mut conn,
            user_id,
            req.currency,
        )?;

        if wallet.balance < amount_minor {
            return Err(ApiError::Payment("Insufficient balance".into()));
        }

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

        // External transfer
        state
            .paystack
            .initiate_transfer(recipient_code, amount_minor, &req.reference.to_string())
            .await?;

        // Atomic DB write
        let tx_id = conn.transaction::<_, ApiError, _>(|conn| {
            WalletRepository::debit(conn, wallet.id, amount_minor)?;

            let tx = TransactionRepository::create(
                conn,
                NewTransaction {
                    user_id,
                    counterparty_id: None,
                    intent: TransactionIntent::Payout,
                    amount: amount_minor,
                    currency: wallet.currency,
                    txn_state: PaymentState::Completed, // Simplified
                    provider: Some(PaymentProvider::Paystack),
                    provider_reference: Some(recipient_code),
                    idempotency_key: &req.idempotency_key,
                    reference: req.reference,
                    description: Some("Wallet withdrawal"),
                    metadata: json!({
                        "bank_code": bank_account.bank_code,
                        "account_number": bank_account.account_number,
                    }),
                },
            )?;

            // Ledger
            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: tx.id,
                    amount: -amount_minor,
                },
            )?;

            Ok::<Uuid, ApiError>(tx.id)
        })?;

        let _ = AuditService::log_event(
            state,
            Some(user_id),
            "withdrawal.initiated",
            Some("transaction"),
            Some(&tx_id.to_string()),
            json!({
                "amount": amount_minor,
                "currency": wallet.currency,
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
