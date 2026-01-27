use axum::body::Bytes;
use diesel::Connection;

use hmac::KeyInit;
use http::HeaderMap;
pub use payego_primitives::{
    error::ApiError,
    models::{
        app_state::AppState,
        dtos::providers_dto::PaystackWebhook,
        entities::enum_types::{PaymentState, TransactionIntent},
        transaction::Transaction,
        wallet_ledger::NewWalletLedger,
    },
};
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
use secrecy::ExposeSecret;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct PaystackService;

impl PaystackService {
    pub fn handle_event(
        state: Arc<AppState>,
        headers: HeaderMap,
        body: &Bytes,
    ) -> Result<(), ApiError> {
        let signature = headers
            .get("x-paystack-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Payment("Missing Paystack signature".into()))?;

        PaystackService::verify_paystack_signature(
            state
                .config
                .paystack_details
                .paystack_webhook_secret
                .expose_secret(),
            body,
            signature,
        )?;

        let payload: PaystackWebhook = serde_json::from_slice(body)
            .map_err(|_| ApiError::Payment("Invalid webhook payload".into()))?;

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let event = payload.event.as_str();

        if !matches!(event, "transfer.success" | "transfer.failed") {
            return Ok(());
        }

        let reference = Uuid::parse_str(&payload.data.reference)
            .map_err(|_| ApiError::Payment("Invalid transaction reference".into()))?;

        conn.transaction(|conn| {
            let tx = TransactionRepository::find_by_id_or_reference(conn, reference)?
                .ok_or(ApiError::Payment("Transaction not found".into()))?;

            // 🔒 Idempotency guard
            if !matches!(tx.txn_state, PaymentState::Pending) {
                info!("Ignoring duplicate webhook for {}", reference);
                return Ok(());
            }

            match event {
                "transfer.success" => {
                    TransactionRepository::update_state(conn, tx.id, PaymentState::Completed)?;
                }

                "transfer.failed" => {
                    TransactionRepository::update_state(conn, tx.id, PaymentState::Failed)?;

                    // 💰 Refund ONLY for payout intents
                    if matches!(tx.intent, TransactionIntent::Payout) {
                        let amount_to_refund = tx.amount.abs();
                        // 💰 Wallet Credit
                        WalletRepository::credit_by_user_and_currency(conn, tx.user_id, tx.currency, amount_to_refund)?;

                        // To get the wallet_id for ledger entry, we might need to find it or credit returning id
                        // Let's use find_by_user_and_currency_with_lock for safety or just find_by_user_and_currency
                        let wallet = WalletRepository::find_by_user_and_currency(conn, tx.user_id, tx.currency)?
                            .ok_or(ApiError::Internal("Wallet not found for refund".into()))?;

                        // 📝 Ledger Entry (Refund)
                        WalletRepository::add_ledger_entry(conn, NewWalletLedger {
                            wallet_id: wallet.id,
                            transaction_id: tx.id,
                            amount: amount_to_refund,
                        })?;
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
