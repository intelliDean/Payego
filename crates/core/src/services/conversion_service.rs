use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use payego_primitives::error::ApiError;
use payego_primitives::models::{
    AppState, ConvertRequest, ConvertResponse, NewTransaction, Transaction, Wallet,
};
use payego_primitives::schema::{transactions, wallets};
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::LazyLock;
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
        state: &AppState,
        user_id: Uuid,
        req: ConvertRequest,
    ) -> Result<ConvertResponse, ApiError> {
        info!("Convert currency initiated");

        if req.from_currency == req.to_currency {
            return Err(ApiError::Payment(
                "From and to currencies must be different".to_string(),
            ));
        }

        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let existing_transaction = transactions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ")
                    .bind::<diesel::sql_types::Text, _>(&req.idempotency_key),
            )
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(|e: diesel::result::Error| {
                error!("Database error checking idempotency: {}", e);
                ApiError::from(e)
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

        let exchange_rate = Self::get_exchange_rate(
            &state.exchange_api_url,
            &req.from_currency,
            &req.to_currency,
        )
        .await?;

        let fee = 0.01 * req.amount * exchange_rate;
        let amount_cents = (req.amount * 100.0).round() as i64;

        let transaction_reference = Uuid::new_v4();
        let (final_converted_amount, final_fee) = conn.transaction::<_, ApiError, _>(|conn| {
            let from_wallet = wallets::table
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.from_currency))
                .first::<Wallet>(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Source wallet lookup failed: {}", e);
                    ApiError::from(e)
                })?;

            if from_wallet.balance < amount_cents {
                return Err(ApiError::Payment("Insufficient balance".to_string()));
            }

            let to_wallet = wallets::table
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.to_currency))
                .first::<Wallet>(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Destination wallet lookup failed: {}", e);
                    ApiError::from(e)
                })?;

            let final_converted_amount = (req.amount * exchange_rate) - fee;
            let final_converted_amount_cents = (final_converted_amount * 100.0).round() as i64;

            diesel::update(wallets::table)
                .filter(wallets::id.eq(from_wallet.id))
                .set(
                    wallets::balance
                        .eq(sql::<BigInt>("balance - ").bind::<BigInt, _>(amount_cents)),
                )
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Source wallet debit failed: {}", e);
                    ApiError::from(e)
                })?;

            diesel::update(wallets::table)
                .filter(wallets::id.eq(to_wallet.id))
                .set(wallets::balance.eq(
                    sql::<BigInt>("balance + ").bind::<BigInt, _>(final_converted_amount_cents),
                ))
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Destination wallet credit failed: {}", e);
                    ApiError::from(e)
                })?;

            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: -amount_cents,
                    transaction_type: "currency_conversion".to_string(),
                    status: "completed".to_string(),
                    provider: None,
                    description: Some(format!(
                        "Conversion from {} to {}",
                        req.from_currency, req.to_currency
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
                .map_err(|e: diesel::result::Error| {
                    error!("Transaction insert failed: {}", e);
                    ApiError::from(e)
                })?;

            Ok((final_converted_amount, fee))
        })?;

        info!(
            "Currency conversion completed: user_id={}, amount={}, from={}, to={}",
            user_id, req.amount, req.from_currency, req.to_currency
        );

        Ok(ConvertResponse {
            transaction_id: transaction_reference.to_string(),
            converted_amount: final_converted_amount,
            exchange_rate,
            fee: final_fee,
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
        let resp = client.get(url).send().await.map_err(|e: reqwest::Error| {
            ApiError::Payment(format!("Exchange rate API error: {}", e))
        })?;

        let status = resp.status();
        let body = resp.json::<Value>().await.map_err(|e: reqwest::Error| {
            ApiError::Payment(format!("Invalid exchange rate response: {}", e))
        })?;

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
