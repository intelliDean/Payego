use diesel::prelude::*;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::AppState,
        entities::enum_types::PaymentState,
        enum_types::{CurrencyCode, TransactionIntent},
        providers_dto::StripeWebhookContext,
        transaction::Transaction,
        transaction_dto::{TransactionResponse, TransactionSummaryDto, TransactionsResponse},
    },
    schema::{transactions, wallets},
};
use tracing::{error, info, warn};
use uuid::Uuid;

const RECENT_TX_LIMIT: i64 = 5;

pub struct TransactionService;

impl TransactionService {
    pub fn apply_stripe_webhook(
        state: &AppState,
        ctx: StripeWebhookContext,
    ) -> Result<(), ApiError> {

        let mut conn = state.db.get().map_err(|e| {
            tracing::error!("DB connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        conn.transaction(|conn| {
            let tx = transactions::table
                .filter(transactions::reference.eq(ctx.transaction_ref))
                .for_update()
                .first::<Transaction>(conn)
                .optional()?
                .ok_or(ApiError::Payment("Transaction not found".into()))?;

            // 🔒 Idempotency
            if tx.txn_state == PaymentState::Completed {
                info!("Stripe webhook already processed for transaction reference: {}", tx.reference);
                return Ok(());
            }

            info!("Processing Stripe webhook for transaction Reference: {}, Currency: {}, ID: {}", tx.reference, tx.currency, tx.id);
            // 🧪 Currency validation
            if ctx.currency != tx.currency.to_string() {
                error!("Stripe Webhook Currency Mismatch: Event Currency={}, DB Transaction Currency={}", ctx.currency, tx.currency.to_string());
                return Err(ApiError::Payment("Currency mismatch".into()));
            }

            diesel::update(transactions::table.find(tx.id))
                .set((
                    transactions::txn_state.eq(PaymentState::Completed),
                    transactions::provider_reference.eq(Some(ctx.provider_reference)),
                    transactions::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(conn)?;

            // 💰 Update Wallet Balance
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(tx.user_id))
                .filter(wallets::currency.eq(tx.currency))
                .set(wallets::balance.eq(wallets::balance + tx.amount as i64))
                .execute(conn)?;

            Ok(())
        })
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
            .filter(transactions::reference.eq(transaction_id))
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
