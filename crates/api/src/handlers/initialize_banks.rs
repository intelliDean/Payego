use payego_primitives::error::ApiError;
use payego_primitives::models::Bank;
use payego_primitives::schema::banks;
use payego_primitives::models::AppState;
use axum::extract::State;
use diesel::prelude::*;
use http::StatusCode;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info};

#[utoipa::path(
    post,
    path = "/api/bank/init",
    responses(
        (status = 201, description = "Banks initialized successfully",),
        (status = 400, description = "Bank initialization failed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn initialize_banks(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.db.get().map_err(|e: diesel::r2d2::PoolError| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Check if banks table is populated
    let bank_count: i64 =
        banks::table
            .count()
            .get_result(&mut conn)
            .map_err(|e: diesel::result::Error| {
                error!("Failed to count banks: {}", e);
                ApiError::from(e)
            })?;

    // Assume at least 10 banks for a valid population (Paystack typically returns ~25 banks for Nigeria)
    const MIN_BANKS: i64 = 10;
    if bank_count >= MIN_BANKS {
        info!(
            "Banks table already populated with {} banks, skipping Paystack fetch",
            bank_count
        );
        return Ok(StatusCode::OK);
    }

    // Fetch from Paystack
    let paystack_key = state.paystack_secret_key.expose_secret();
    let client = Client::new();
    let url = "https://api.paystack.co/bank?country=nigeria";
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", paystack_key))
        .send()
        .await
        .map_err(|e: reqwest::Error| {
            error!("Paystack banks API error: {}", e);
            ApiError::Payment(format!("Paystack banks API error: {}", e))
        })?;

    let status = resp.status();
    let body = resp.json::<Value>().await.map_err(|e: reqwest::Error| {
        error!("Paystack response parsing error: {}", e);
        ApiError::Payment(format!("Paystack response error: {}", e))
    })?;

    if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
        let message = body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack error")
            .to_string();
        error!("Paystack banks fetch failed: {}", message);
        return Err(ApiError::Payment(format!("Paystack banks fetch failed: {}", message)));
    }

    let banks_data = body["data"].as_array().ok_or_else(|| {
        error!("Invalid Paystack response: missing banks data");
        ApiError::Payment("Invalid Paystack response".to_string())
    })?;

    let mut banks: Vec<Bank> = Vec::new();
    let mut skipped = 0;
    for bank in banks_data.iter() {
        let id = bank["id"].as_i64();
        let name = bank["name"].as_str().map(|s| s.to_string());
        let code = bank["code"].as_str().map(|s| s.to_string());
        let currency = bank["currency"].as_str().map(|s| s.to_string());
        let country = bank["country"].as_str().map(|s| s.to_string());
        let gateway = bank["gateway"].as_str().map(|s| s.to_string());
        let pay_with_bank = bank["pay_with_bank"].as_bool();
        let is_active = bank["is_active"].as_bool();

        match (id, name, code, currency, country) {
            (Some(id), Some(name), Some(code), Some(currency), Some(country)) => {
                banks.push(Bank {
                    id,
                    name,
                    code,
                    currency,
                    country,
                    gateway,
                    pay_with_bank,
                    is_active,
                });
            }
            _ => {
                error!("Invalid bank data: {:?}", bank);
                skipped += 1;
            }
        }
    }

    if banks.is_empty() {
        error!("No valid banks fetched from Paystack");
        return Err(ApiError::Payment("No valid banks fetched from Paystack".to_string()));
    }

    // Insert banks into database with ON CONFLICT DO NOTHING
    let inserted_count = diesel::insert_into(banks::table)
        .values(&banks)
        .on_conflict(banks::code)
        .do_nothing()
        .execute(&mut conn)
        .map_err(|e: diesel::result::Error| {
            error!("Failed to insert banks into database: {}", e);
            ApiError::from(e)
        })?;

    info!(
        "Inserted {} banks into database during startup, skipped {} ({} invalid, {} duplicates)",
        inserted_count,
        banks.len() - inserted_count + skipped,
        skipped,
        banks.len() - inserted_count
    );
    Ok(StatusCode::OK)
}
