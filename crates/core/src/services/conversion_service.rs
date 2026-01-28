use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
pub use crate::app_state::AppState;
pub use crate::security::Claims;
use diesel::prelude::*;
pub use payego_primitives::{
    error::ApiError,
    models::{
        dtos::wallet_dto::{ConvertRequest, ConvertResponse},
        enum_types::{CurrencyCode, PaymentProvider, PaymentState, TransactionIntent},
        transaction::NewTransaction,
    },
    schema::{transactions, wallet_ledger, wallets},
};
use serde_json::json;
use uuid::Uuid;

pub struct ConversionService;

impl ConversionService {
    pub async fn convert_currency(
        state: &AppState,
        user_id: Uuid,
        req: ConvertRequest,
    ) -> Result<ConvertResponse, ApiError> {
        if req.from_currency == req.to_currency {
            return Err(ApiError::Payment("Currencies must differ".into()));
        }

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        // ---------- IDEMPOTENCY ----------
        if let Some(tx) = TransactionRepository::find_by_idempotency_key(
            &mut conn,
            user_id,
            &req.idempotency_key,
        )? {
            let get_i64 = |key: &str| {
                tx.metadata
                    .get(key)
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ApiError::Internal(format!("Missing/invalid {}", key)))
            };

            return Ok(ConvertResponse {
                transaction_id: tx.reference.to_string(),
                converted_amount: get_i64("converted_amount_cents")? as f64 / 100.0,
                exchange_rate: get_i64("exchange_rate_scaled")? as f64 / 1_000_000.0,
                fee: get_i64("fee_cents")? as f64 / 100.0,
            });
        }

        // ---------- RATE ----------
        let rate = state.fx.get_rate(req.from_currency, req.to_currency).await?;

        if !(0.0001..10_000.0).contains(&rate) {
            return Err(ApiError::Payment("Suspicious exchange rate".into()));
        }

        let rate_scaled = (rate * 1_000_000.0).round() as i64;
        let converted_cents = req.amount_cents * rate_scaled / 1_000_000;
        let fee_bps = state.config.conversion_fee_bps;
        let fee_cents = (converted_cents as i128 * fee_bps / 10_000) as i64;
        let net_cents = converted_cents - fee_cents;

        let tx_ref = Uuid::new_v4();

        conn.transaction::<_, ApiError, _>(|conn| {
            let from_wallet = WalletRepository::find_by_user_and_currency_with_lock(
                conn,
                user_id,
                req.from_currency,
            )?;
            let to_wallet = WalletRepository::create_if_not_exists(conn, user_id, req.to_currency)?;

            if from_wallet.balance < req.amount_cents {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            let tx = TransactionRepository::create(
                conn,
                NewTransaction {
                    user_id,
                    counterparty_id: None,
                    intent: TransactionIntent::Conversion,
                    amount: req.amount_cents,
                    currency: req.from_currency,
                    txn_state: PaymentState::Completed,
                    provider: Some(PaymentProvider::Internal),
                    provider_reference: None,
                    idempotency_key: &req.idempotency_key,
                    reference: tx_ref,
                    description: Some("Currency conversion"),
                    metadata: json!({
                        "exchange_rate_scaled": rate_scaled,
                        "converted_amount_cents": net_cents,
                        "fee_cents": fee_cents,
                        "quoted_at": chrono::Utc::now()
                    }),
                },
            )?;

            WalletRepository::add_ledger_entry(
                conn,
                payego_primitives::models::wallet_ledger::NewWalletLedger {
                    wallet_id: from_wallet.id,
                    transaction_id: tx.id,
                    amount: -req.amount_cents,
                },
            )?;

            WalletRepository::add_ledger_entry(
                conn,
                payego_primitives::models::wallet_ledger::NewWalletLedger {
                    wallet_id: to_wallet.id,
                    transaction_id: tx.id,
                    amount: net_cents,
                },
            )?;

            WalletRepository::debit(conn, from_wallet.id, req.amount_cents)?;
            WalletRepository::credit(conn, to_wallet.id, net_cents)?;

            Ok(())
        })?;

        Ok(ConvertResponse {
            transaction_id: tx_ref.to_string(),
            converted_amount: net_cents as f64 / 100.0,
            exchange_rate: rate,
            fee: fee_cents as f64 / 100.0,
        })
    }

    pub async fn get_exchange_rate(
        state: &AppState,
        from: CurrencyCode,
        to: CurrencyCode,
    ) -> Result<f64, ApiError> {
        state.fx.get_rate(from, to).await
    }
}
