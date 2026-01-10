use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use hmac::KeyInit;
use hmac::{Hmac, Mac};
use payego_primitives::error::ApiError;
use payego_primitives::models::AppState;
use serde_json::Value;
use sha2::Sha256;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

// https://e02e3895d11f.ngrok-free.app/webhooks/paystack
// Type alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

#[utoipa::path(
    post,
    path = "/webhooks/paystack",
    request_body = Value,
    responses(
        (status = 200, description = "Webhook processed"),
        (status = 400, description = "Invalid webhook or signature"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Webhook"
)]
pub async fn paystack_webhook(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> Result<StatusCode, ApiError> {
    // Validate Paystack webhook signature
    let signature = headers
        .get("x-paystack-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            error!("Missing or invalid Paystack signature header");
            ApiError::Payment("Missing or invalid signature".to_string())
        })?;

    let paystack_secret = std::env::var("PAYSTACK_WEBHOOK_SECRET").map_err(|_| {
        error!("PAYSTACK_WEBHOOK_SECRET not set");
        ApiError::Token("Paystack webhook secret not set".to_string())
    })?;

    let payload_bytes = serde_json::to_vec(&payload).map_err(|e: serde_json::Error| {
        error!("Failed to serialize payload: {}", e);
        ApiError::Payment("Invalid webhook payload".to_string())
    })?;

    let mut mac = HmacSha256::new_from_slice(paystack_secret.as_bytes())
        .map_err(|_| ApiError::Token("Invalid webhook secret".to_string()))?;
    mac.update(&payload_bytes);
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    if expected_signature != signature {
        error!(
            "Invalid Paystack signature: expected {}, got {}",
            expected_signature, signature
        );
        return Err(ApiError::Payment("Invalid webhook signature".to_string()));
    }

    // Extract event
    let event = payload["event"].as_str().ok_or_else(|| {
        error!("Missing or invalid event in payload");
        ApiError::Payment("Invalid event".to_string())
    })?;
    debug!("Received Paystack webhook event: {}", event);

    // Get database connection
    let conn = &mut state.db.get().map_err(|e: diesel::r2d2::PoolError| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Handle events
    match event {
        "transfer.success" | "transfer.failed" => {
            let reference = payload["data"]["reference"].as_str().ok_or_else(|| {
                error!("Missing reference in payload");
                ApiError::Payment("Missing reference".to_string())
            })?;
            let transaction_id = Uuid::parse_str(reference).map_err(|e: uuid::Error| {
                error!("Invalid transaction ID: {}", e);
                ApiError::Payment("Invalid transaction ID".to_string())
            })?;

            // Find transaction
            let transaction = payego_primitives::schema::transactions::table
                .filter(payego_primitives::schema::transactions::reference.eq(transaction_id))
                .select(Transaction::as_select())
                .first(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Transaction lookup failed: {}", e);
                    if e.to_string().contains("not found") {
                        ApiError::Payment("Transaction not found".to_string())
                    } else {
                        ApiError::from(e)
                    }
                })?;

            // Check if already processed
            if transaction.status == "completed" || transaction.status == "failed" {
                info!("Transaction already processed: reference={}", reference);
                return Ok(StatusCode::OK);
            }

            // Update transaction status
            let new_status = if event == "transfer.success" {
                "completed"
            } else {
                "failed"
            };

            diesel::update(payego_primitives::schema::transactions::table.find(transaction_id))
                .set(payego_primitives::schema::transactions::status.eq(new_status))
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Transaction update failed: {}", e);
                    ApiError::from(e)
                })?;

            info!(
                "Paystack webhook processed: event={}, reference={}, new_status={}",
                event, reference, new_status
            );

            // If transfer failed, reverse wallet balance (for paystack_payout)
            if event == "transfer.failed" && transaction.transaction_type == "paystack_payout" {
                let user_id = transaction.user_id;
                let amount = transaction.amount.abs(); // Amount is negative in paystack_payout
                let currency = payload["data"]["currency"].as_str().ok_or_else(|| {
                    error!("Missing currency in payload");
                    ApiError::Payment("Missing currency".to_string())
                })?;

                diesel::update(payego_primitives::schema::wallets::table)
                    .filter(payego_primitives::schema::wallets::user_id.eq(user_id))
                    .filter(payego_primitives::schema::wallets::currency.eq(currency))
                    .set(
                        payego_primitives::schema::wallets::balance.eq(diesel::dsl::sql::<
                            diesel::sql_types::BigInt,
                        >(
                            "balance + "
                        )
                        .bind::<diesel::sql_types::BigInt, _>(amount)),
                    )
                    .execute(conn)
                    .map_err(|e: diesel::result::Error| {
                        error!("Wallet update failed: {}", e);
                        ApiError::from(e)
                    })?;

                info!(
                    "Reversed wallet balance: user_id={}, amount={}, currency={}",
                    user_id, amount, currency
                );
            }
        }
        _ => {
            debug!("Ignored Paystack event: {}", event);
            return Ok(StatusCode::OK);
        }
    }

    Ok(StatusCode::OK)
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = payego_primitives::schema::transactions)]
pub struct Transaction {
    pub user_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub transaction_type: String,
}
