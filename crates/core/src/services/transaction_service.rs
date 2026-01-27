use crate::repositories::transaction_repository::TransactionRepository;
use diesel::prelude::*;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::AppState,
        entities::enum_types::PaymentState,
        enum_types::{CurrencyCode, TransactionIntent},
        transaction::Transaction,
        transaction_dto::{TransactionResponse, TransactionSummaryDto, TransactionsResponse},
        wallet_ledger::{NewWalletLedger, WalletLedger},
        wallet::Wallet,
    },
    schema::{transactions, wallet_ledger, wallets},
};
use crate::repositories::wallet_repository::WalletRepository;
use stripe::PaymentIntent;
use tracing::{error, info, warn};
use uuid::Uuid;

const RECENT_TX_LIMIT: i64 = 5;

pub struct TransactionService;

impl TransactionService {
    pub fn handle_payment_intent_succeeded(
        state: &AppState,
        pay_int: PaymentIntent,
    ) -> Result<(), ApiError> {
        let tx_ref = pay_int
            .metadata
            .get("transaction_reference")
            .ok_or(ApiError::Payment("Missing transaction_reference".into()))?;

        let transaction_ref = Uuid::parse_str(tx_ref)
            .map_err(|_| ApiError::Payment("Invalid transaction_reference".into()))?;

        let amount = pay_int.amount_received;
        let currency = pay_int.currency.to_string().to_uppercase();
        let provider_reference = pay_int.id.clone();

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        conn.transaction(|conn| {
            let tx = TransactionRepository::find_by_reference_for_update(conn, transaction_ref)?
                .ok_or(ApiError::Payment("Transaction not found".into()))?;

            // 🔒 Idempotency
            if tx.txn_state == PaymentState::Completed {
                info!("Transaction already completed: {}", tx.reference);
                return Ok(());
            }

            // 🧪 Currency check
            if tx.currency.to_string() != currency {
                return Err(ApiError::Payment("Currency mismatch".into()));
            }

            // Update transaction
            TransactionRepository::update_status_and_provider_ref(
                conn,
                tx.id,
                PaymentState::Completed,
                Some(provider_reference.to_string()),
            )?;

            // 💰 Wallet UPSERT (critical)
            let wallet_id = WalletRepository::upsert_balance(conn, tx.user_id, tx.currency, amount)?;

            // 📝 Ledger Entry
            WalletRepository::add_ledger_entry(conn, NewWalletLedger {
                wallet_id,
                transaction_id: tx.id,
                amount,
            })?;

            Ok(())
        })
    }
    pub fn handle_payment_intent_failed(
        state: &AppState,
        pi: PaymentIntent,
    ) -> Result<(), ApiError> {
        let tx_ref = match pi.metadata.get("transaction_reference") {
            Some(v) => v,
            None => return Ok(()), // nothing to do
        };

        let transaction_ref = Uuid::parse_str(tx_ref).ok();

        if let Some(tx_ref) = transaction_ref {
            let mut conn = state
                .db
                .get()
                .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
            TransactionRepository::update_status_by_reference(
                &mut conn,
                tx_ref,
                PaymentState::Failed,
            )?;
        }

        Ok(())
    }

    pub fn handle_payment_intent_canceled(
        state: &AppState,
        pi: PaymentIntent,
    ) -> Result<(), ApiError> {
        let tx_ref = match pi.metadata.get("transaction_reference") {
            Some(v) => v,
            None => return Ok(()),
        };

        let transaction_ref = Uuid::parse_str(tx_ref).ok();

        if let Some(tx_ref) = transaction_ref {
            let mut conn = state
                .db
                .get()
                .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
            TransactionRepository::update_status_by_reference(
                &mut conn,
                tx_ref,
                PaymentState::Cancelled,
            )?;
        }

        Ok(())
    }

    pub async fn get_user_transaction(
        state: &AppState,
        claims: &Claims,
        transaction_id: Uuid,
    ) -> Result<TransactionResponse, ApiError> {
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            warn!("txn.fetch: invalid user id in claims");
            ApiError::Auth(AuthError::InvalidToken("Invalid token".into()))
        })?;

        let mut conn = state.db.get().map_err(|_| {
            error!("txn.fetch: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let tx =
            TransactionRepository::find_by_id_or_ref_and_user(&mut conn, transaction_id, user_id)?
                .ok_or_else(|| ApiError::Internal("Transaction not found".into()))?;

        Ok(tx.into())
    }

    pub async fn recent_transactions(
        state: &AppState,
        uid: Uuid,
    ) -> Result<TransactionsResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("transactions.recent: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let rows = TransactionRepository::find_recent_by_user(&mut conn, uid, RECENT_TX_LIMIT)
            .map_err(|_| {
                error!("transactions.recent: failed to load transactions");
                ApiError::Internal("Failed to load transactions".into())
            })?;

        Ok(TransactionsResponse { transactions: rows })
    }
}
