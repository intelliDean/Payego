use diesel::prelude::*;
use tracing::info;

use payego_primitives::{
    error::ApiError, models::entities::enum_types::PaymentState, models::transaction::Transaction,
    schema::transactions,
};

use crate::services::stripe_service::StripeWebhookContext;

pub struct TransactionService;

impl TransactionService {
    pub fn apply_stripe_webhook(
        conn: &mut PgConnection,
        ctx: StripeWebhookContext,
    ) -> Result<(), ApiError> {
        conn.transaction(|conn| {
            let tx = transactions::table
                .filter(transactions::reference.eq(ctx.transaction_ref))
                .for_update()
                .first::<Transaction>(conn)
                .optional()?
                .ok_or(ApiError::Payment("Transaction not found".into()))?;

            // 🔒 Idempotency
            if tx.txn_state == PaymentState::Completed {
                info!("Stripe webhook already processed: {}", tx.reference);
                return Ok(());
            }

            // 🧪 Currency validation
            if ctx.currency != tx.currency.to_string() {
                return Err(ApiError::Payment("Currency mismatch".into()));
            }

            diesel::update(transactions::table.find(tx.id))
                .set((
                    transactions::txn_state.eq(PaymentState::Completed),
                    transactions::provider_reference.eq(Some(ctx.provider_reference)),
                    transactions::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(conn)?;

            Ok(())
        })
    }
}
