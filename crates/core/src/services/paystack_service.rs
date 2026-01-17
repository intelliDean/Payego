use chrono::{DateTime, Utc};
use diesel::prelude::*;
use hmac::{KeyInit,};
use payego_primitives::{
    error::ApiError,
    models::{
        entities::enum_types::{PaymentState, TransactionIntent},
        transaction::Transaction,
    },
    schema::{transactions, wallets},
};
use tracing::{info};
use uuid::Uuid;
use payego_primitives::models::dtos::dtos::PaystackWebhook;

pub struct PaystackService;

impl PaystackService {
    pub fn handle_event(
        conn: &mut PgConnection,
        payload: &PaystackWebhook,
    ) -> Result<(), ApiError> {
        let event = payload.event.as_str();

        if !matches!(event, "transfer.success" | "transfer.failed") {
            return Ok(());
        }

        let reference = Uuid::parse_str(&payload.data.reference)
            .map_err(|_| ApiError::Payment("Invalid transaction reference".into()))?;

        conn.transaction(|conn| {
            let tx = transactions::table
                .filter(transactions::reference.eq(reference))
                .first::<Transaction>(conn)
                .optional()?
                .ok_or(ApiError::Payment("Transaction not found".into()))?;

            // 🔒 Idempotency guard
            if !matches!(tx.txn_state, PaymentState::Pending) {
                info!("Ignoring duplicate webhook for {}", reference);
                return Ok(());
            }

            match event {
                "transfer.success" => {
                    diesel::update(transactions::table.find(tx.id))
                        .set(transactions::txn_state.eq(PaymentState::Completed))
                        .execute(conn)?;
                }

                "transfer.failed" => {
                    diesel::update(transactions::table.find(tx.id))
                        .set(transactions::txn_state.eq(PaymentState::Failed))
                        .execute(conn)?;

                    // 💰 Refund ONLY for payout intents
                    if matches!(tx.intent, TransactionIntent::Payout) {
                        diesel::update(wallets::table)
                            .filter(wallets::user_id.eq(tx.user_id))
                            .filter(wallets::currency.eq(tx.currency))
                            .set(wallets::balance.eq(wallets::balance + tx.amount.abs()))
                            .execute(conn)?;
                    }
                }

                _ => {}
            }

            Ok(())
        })
    }

    pub fn verify_paystack_signature(
        secret: &str,
        payload: &[u8],
        actual_signature: &str,
    ) -> Result<(), ApiError> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        use subtle::ConstantTimeEq;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| ApiError::Token("Invalid webhook secret".into()))?;

        mac.update(payload);
        let expected = hex::encode(mac.finalize().into_bytes());

        if expected
            .as_bytes()
            .ct_eq(actual_signature.as_bytes())
            .unwrap_u8()
            != 1
        {
            return Err(ApiError::Payment("Invalid Paystack signature".into()));
        }

        Ok(())
    }
}

