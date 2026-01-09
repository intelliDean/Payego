use crate::error::ApiError;
use crate::handlers::internal_conversion::{ConvertRequest, ConvertResponse};
use crate::models::models::{AppState, NewTransaction, Transaction, Wallet};
use crate::schema::{transactions, wallets};
use diesel::prelude::*;
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::{Arc, LazyLock};
use tracing::{error, info};
use uuid::Uuid;

static SUPPORTED_CURRENCIES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(USD|NGN|GBP|EUR|CAD|AUD|JPY|CHF|CNY|SEK|NZD|MXN|SGD|HKD|NOK|KRW|TRY|INR|BRL|ZAR)$",
    )
    .expect("Invalid currency regex")
});

pub struct ConversionService;

impl ConversionService {
    pub async fn convert_currency(
        state: Arc<AppState>,
        user_id: Uuid,
        req: ConvertRequest,
    ) -> Result<ConvertResponse, ApiError> {
        info!("Convert currency initiated");

        // Validate currencies
        if req.from_currency == req.to_currency {
            return Err(ApiError::Payment(
                "From and to currencies must be different".to_string(),
            ));
        }

        // Get database connection
        let mut conn = state.db.get().map_err(|e| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // Idempotency check with metadata
        let existing_transaction = transactions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ")
                    .bind::<diesel::sql_types::Text, _>(&req.idempotency_key),
            )
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error checking idempotency: {}", e);
                ApiError::Database(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!(
                "Idempotent request: transaction {} already exists for key {}",
                tx.reference, req.idempotency_key
            );
            let metadata = tx.metadata.as_ref().ok_or(ApiError::Payment(
                "Missing metadata in existing transaction".to_string(),
            ))?;

            let converted_amount = metadata["converted_amount"].as_f64().unwrap_or(0.0);
            let exchange_rate = metadata["exchange_rate"].as_f64().unwrap_or(0.0);
            let fee = metadata["fee"].as_f64().unwrap_or(0.0);

            return Ok(ConvertResponse {
                transaction_id: tx.reference.to_string(),
                converted_amount,
                exchange_rate,
                fee,
            });
        }

        // Fetch exchange rate
        let exchange_rate = Self::get_exchange_rate(
            &state.exchange_api_url,
            &req.from_currency,
            &req.to_currency,
        )
        .await?;

        // Calculate fee (e.g., 1%)
        let fee = 0.01 * req.amount * exchange_rate;
        let fee_cents = (fee * 100.0).round() as i64;
        let amount_cents = (req.amount * 100.0).round() as i64;

        // Atomic transaction
        let transaction_reference = Uuid::new_v4();
        let converted_amount = conn.transaction(|conn| {
            // Fetch sender wallet
            let from_wallet = wallets::table
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.from_currency))
                .select(Wallet::as_select())
                .first(conn)
                .map_err(|e| {
                    error!("From wallet lookup failed: {}", e);
                    if e.to_string().contains("not found") {
                        ApiError::Payment(format!("Wallet not found for {}", req.from_currency))
                    } else {
                        ApiError::Database(e)
                    }
                })?;

            // Validate balance
            if from_wallet.balance < amount_cents {
                error!(
                    "Insufficient balance: available={}, required={}",
                    from_wallet.balance, amount_cents
                );
                return Err(ApiError::Payment("Insufficient balance".to_string()));
            }

            // Calculate converted amount in cents
            let converted_cents = ((amount_cents as f64) * exchange_rate).round() as i64;
            let net_converted_cents = converted_cents - fee_cents;
            let final_converted_amount = net_converted_cents as f64 / 100.0;

            // Debit from_wallet
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.from_currency))
                .set(wallets::balance.eq(wallets::balance - amount_cents))
                .execute(conn)
                .map_err(|e| {
                    error!("From wallet update failed: {}", e);
                    ApiError::Database(e)
                })?;

            // Credit to_wallet
            diesel::insert_into(wallets::table)
                .values((
                    wallets::user_id.eq(user_id),
                    wallets::balance.eq(net_converted_cents),
                    wallets::currency.eq(&req.to_currency),
                ))
                .on_conflict((wallets::user_id, wallets::currency))
                .do_update()
                .set(wallets::balance.eq(wallets::balance + net_converted_cents))
                .execute(conn)
                .map_err(|e| {
                    error!("To wallet update failed: {}", e);
                    ApiError::Database(e)
                })?;

            // Insert transaction
            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: amount_cents,
                    transaction_type: "currency_conversion".to_string(),
                    status: "completed".to_string(),
                    provider: Some("internal".to_string()),
                    description: Some(format!(
                        "Converted {} {} to {} {} (rate: {}, fee: {})",
                        req.amount,
                        req.from_currency,
                        final_converted_amount,
                        req.to_currency,
                        exchange_rate,
                        fee
                    )),
                    reference: transaction_reference,
                    currency: req.from_currency.clone(),
                    metadata: Some(json!({
                        "idempotency_key": req.idempotency_key,
                        "converted_amount": final_converted_amount,
                        "exchange_rate": exchange_rate,
                        "fee": fee
                    })),
                })
                .execute(conn)
                .map_err(|e| {
                    error!("Transaction insert failed: {}", e);
                    ApiError::Database(e)
                })?;

            Ok(final_converted_amount)
        })?;

        info!(
            "Currency conversion completed: user_id={}, amount={}, from={}, to={}",
            user_id, req.amount, req.from_currency, req.to_currency
        );

        Ok(ConvertResponse {
            transaction_id: transaction_reference.to_string(),
            converted_amount,
            exchange_rate,
            fee,
        })
    }

    async fn get_exchange_rate(
        base_url: &str,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<f64, ApiError> {
        if from_currency == to_currency {
            return Ok(1.0);
        }
        let url = format!("{}/{}", base_url, from_currency);
        let client = Client::new();
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("Exchange rate API error: {}", e)))?;

        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e| ApiError::Payment(format!("Invalid exchange rate response: {}", e)))?;

        if !status.is_success() {
            return Err(ApiError::Payment(format!(
                "Exchange rate API failed: {}",
                body["error"].as_str().unwrap_or("Unknown error")
            )));
        }

        let rate = body["rates"][to_currency]
            .as_f64()
            .ok_or_else(|| ApiError::Payment(format!("No rate found for {}", to_currency)))?;

        Ok(rate)
    }
}
