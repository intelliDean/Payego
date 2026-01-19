use diesel::prelude::*;
pub use payego_primitives::{
    error::ApiError,
    config::security_config::Claims,
    models::{
        app_state::app_state::AppState,
        conversion_dto::{ConvertRequest, ExchangeRateResponse},
        dtos::conversion_dto::ConvertResponse,
        enum_types::{
            CurrencyCode, PaymentProvider, PaymentState, TransactionIntent,
        },
        transaction::{NewTransaction, Transaction},
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
    schema::{transactions, wallet_ledger, wallets},
    
};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
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
        if let Some(tx) = transactions::table
            .filter(transactions::user_id.eq(user_id))
            .filter(transactions::idempotency_key.eq(&req.idempotency_key))
            .first::<Transaction>(&mut conn)
            .optional()?
        {
            let meta = tx.metadata.clone();

            //closure to help me convert this
            let get_i64 = |key: &str| {
                meta.get(key)
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
        let rate = Self::get_exchange_rate(
            &state.http_client,
            &state.config.exchange_api_url,
            req.from_currency,
            req.to_currency,
        )
        .await?;

        if !(0.0001..10_000.0).contains(&rate) {
            return Err(ApiError::Payment("Suspicious exchange rate".into()));
        }

        let rate_scaled = (rate * 1_000_000.0).round() as i64;
        let converted_cents = req.amount_cents * rate_scaled / 1_000_000;
        let fee_bps = state.config.conversion_fee_bps; // 1%
        let fee_cents = (converted_cents as i128 * fee_bps / 10_000) as i64;
        let net_cents = converted_cents - fee_cents;

        let tx_ref = Uuid::new_v4();

        conn.transaction::<_, ApiError, _>(|conn| {
            // ---------- LOCK WALLETS ----------
            let from_wallet = wallets::table
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(req.from_currency))
                .for_update()
                .first::<Wallet>(conn)
                .map_err(|_| ApiError::Payment("Source wallet not found".into()))?;

            let to_wallet = wallets::table
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(req.to_currency))
                .for_update()
                .first::<Wallet>(conn)
                .map_err(|_| ApiError::Payment("Destination wallet not found".into()))?;

            // ---------- BALANCE CHECK (CACHED) ----------
            debug_assert!(from_wallet.balance >= 0);
            if from_wallet.balance < req.amount_cents {
                return Err(ApiError::Payment("Insufficient balance".into()));
            }

            // ---------- TRANSACTION ----------
            let tx_id: Uuid = diesel::insert_into(transactions::table)
                .values(NewTransaction {
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
                })
                .on_conflict((transactions::user_id, transactions::idempotency_key))
                .do_nothing()
                .returning(transactions::id)
                .get_result(conn)?;

            // ---------- LEDGER ----------
            diesel::insert_into(wallet_ledger::table)
                .values(NewWalletLedger {
                    wallet_id: from_wallet.id,
                    transaction_id: tx_id,
                    amount: -req.amount_cents,
                    // entry_type: "debit",
                })
                .execute(conn)?;

            diesel::insert_into(wallet_ledger::table)
                .values(NewWalletLedger {
                    wallet_id: to_wallet.id,
                    transaction_id: tx_id,
                    amount: net_cents,
                    // entry_type: "credit",
                })
                .execute(conn)?;

            // ---------- UPDATE CACHED BALANCE ----------
            diesel::update(wallets::table.filter(wallets::id.eq(from_wallet.id)))
                .set(wallets::balance.eq(wallets::balance - req.amount_cents))
                .execute(conn)?;

            diesel::update(wallets::table.filter(wallets::id.eq(to_wallet.id)))
                .set(wallets::balance.eq(wallets::balance + net_cents))
                .execute(conn)?;

            Ok(())
        })?;

        Ok(ConvertResponse {
            transaction_id: tx_ref.to_string(),
            converted_amount: net_cents as f64 / 100.0,
            exchange_rate: rate,
            fee: fee_cents as f64 / 100.0,
        })
    }

    async fn get_exchange_rate(
        client: &Client,
        base_url: &str,
        from: CurrencyCode,
        to: CurrencyCode,
    ) -> Result<f64, ApiError> {
        if from == to {
            return Ok(1.0);
        }

        let url = format!("{}/{}", base_url.trim_end_matches('/'), from);

        let resp = client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("FX API unreachable: {}", e)))?;

        let status = resp.status();
        let body = resp
            .json::<ExchangeRateResponse>()
            .await
            .map_err(|_| ApiError::Payment("Invalid FX response".into()))?;

        if !status.is_success() {
            return Err(ApiError::Payment(
                body.error.unwrap_or_else(|| "FX API error".into()),
            ));
        }

        body.rates
            .get(&format!("{}", to))
            .copied()
            .filter(|r| *r > 0.0)
            .ok_or_else(|| ApiError::Payment("Exchange rate not found".into()))
    }
}









