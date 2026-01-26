use chrono::Utc;
use diesel::prelude::*;
use stripe::PaymentIntent;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::AppState,
        entities::enum_types::PaymentState,
        enum_types::{CurrencyCode, TransactionIntent},
        transaction::Transaction,
        transaction_dto::{TransactionResponse, TransactionSummaryDto, TransactionsResponse},
        wallet_ledger::NewWalletLedger,
    },
    schema::{transactions, wallet_ledger, wallets},
};
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

        let mut conn = state.db.get().map_err(|e| {
            ApiError::DatabaseConnection(e.to_string())
        })?;

        conn.transaction(|conn| {
            let tx = transactions::table
                .filter(transactions::reference.eq(transaction_ref))
                .for_update()
                .first::<Transaction>(conn)
                .optional()?
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
            diesel::update(transactions::table.find(tx.id))
                .set((
                    transactions::txn_state.eq(PaymentState::Completed),
                    transactions::provider_reference.eq(Some(provider_reference.to_string())),
                    transactions::updated_at.eq(Utc::now()),
                ))
                .execute(&mut *conn)?;

            // 💰 Wallet UPSERT (critical)
            let wallet_id = diesel::insert_into(wallets::table)
                .values((
                    wallets::user_id.eq(tx.user_id),
                    wallets::currency.eq(tx.currency),
                    wallets::balance.eq(amount),
                ))
                .on_conflict(diesel::dsl::sql::<diesel::sql_types::Record<(diesel::sql_types::Uuid, payego_primitives::schema::sql_types::CurrencyCode)>>("(user_id, currency)"))
                .do_update()
                .set(wallets::balance.eq(wallets::balance + amount))
                .returning(wallets::id)
                .get_result::<Uuid>(&mut *conn)?;

            // 📝 Ledger Entry
            diesel::insert_into(wallet_ledger::table)
                .values(NewWalletLedger {
                    wallet_id,
                    transaction_id: tx.id,
                    amount,
                })
                .execute(&mut *conn)?;

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
            let mut conn = state.db.get().map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
            diesel::update(transactions::table)
                .filter(transactions::reference.eq(tx_ref))
                .set(transactions::txn_state.eq(PaymentState::Failed))
                .execute(&mut *conn)?;
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
            let mut conn = state.db.get().map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
            diesel::update(transactions::table)
                .filter(transactions::reference.eq(tx_ref))
                .set(transactions::txn_state.eq(PaymentState::Cancelled))
                .execute(&mut *conn)?;
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

        let tx = transactions::table
            .filter(transactions::id.eq(transaction_id))
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(|_| {
                error!("txn.fetch: database query failed");
                ApiError::Internal("Failed to fetch transaction".into())
            })?
            .ok_or_else(|| ApiError::Internal("Transaction not found".into()))?;

        Ok(tx.into())
    }

    pub async fn recent_transactions(
        state: &AppState,
        uid: Uuid,
    ) -> Result<TransactionsResponse, ApiError> {
        use payego_primitives::schema::transactions::dsl::*;

        let mut conn = state.db.get().map_err(|_| {
            error!("transactions.recent: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let rows = transactions
            .filter(user_id.eq(uid))
            .order(created_at.desc())
            .limit(RECENT_TX_LIMIT)
            .select((id, intent, amount, currency, created_at, txn_state))
            .load::<(
                Uuid,
                TransactionIntent,
                i64,
                CurrencyCode,
                chrono::DateTime<chrono::Utc>,
                PaymentState,
            )>(&mut conn)
            .map_err(|_| {
                error!("transactions.recent: failed to load transactions");
                ApiError::Internal("Failed to load transactions".into())
            })?;

        let tnx = rows
            .into_iter()
            .map(
                |(tnx_id, tnx_intent, tnx_amount, tnx_currency, tnx_created_at, state)| {
                    TransactionSummaryDto {
                        id: tnx_id,
                        intent: tnx_intent,
                        amount: tnx_amount,
                        currency: tnx_currency,
                        created_at: tnx_created_at,
                        state,
                    }
                },
            )
            .collect();

        Ok(TransactionsResponse { transactions: tnx })
    }
}
