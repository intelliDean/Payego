use crate::AppState;
use crate::error::ApiError;
// use crate::models::user_models::Transaction;
use crate::schema::transactions;
use crate::schema::wallets;
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
};
use diesel::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::schema::transactions::{amount, description, reference, status};
use crate::schema::wallets::user_id;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub reference: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub amount: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CaptureRequest {
    pub order_id: String,
    pub transaction_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct CaptureResponse {
    pub status: String,
    pub transaction_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/paypal/capture",
    request_body = CaptureRequest,
    responses(
        (status = 200, description = "Order captured successfully", body = CaptureResponse),
        (status = 400, description = "Invalid order ID or transaction ID"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Payments"
)]
pub async fn paypal_capture(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CaptureRequest>,
) -> Result<Json<CaptureResponse>, (StatusCode, String)> {
    let client = Client::new();
    let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").expect("PAYPAL_CLIENT_ID must be set");
    let paypal_secret = std::env::var("PAYPAL_SECRET").expect("PAYPAL_SECRET must be set");

    // Get PayPal access token
    let token_response = client
        .post("https://api-m.sandbox.paypal.com/v1/oauth2/token")
        .basic_auth(&paypal_client_id, Some(&paypal_secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await
        .map_err(|e| ApiError::Payment(format!("Failed to get PayPal token: {}", e)))?;

    let token: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| ApiError::Payment(format!("Failed to parse PayPal token: {}", e)))?;
    let access_token = token["access_token"]
        .as_str()
        .ok_or(ApiError::Payment("Missing access_token".to_string()))?;

    // Capture the order
    let capture_response = client
        .post(&format!(
            "https://api-m.sandbox.paypal.com/v2/checkout/orders/{}/capture",
            payload.order_id
        ))
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| ApiError::Payment(format!("PayPal capture failed: {}", e)))?;

    let capture_result: serde_json::Value = capture_response
        .json()
        .await
        .map_err(|e| ApiError::Payment(format!("Failed to parse capture response: {}", e)))?;

    let conn = &mut state
        .db
        .get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
    let trans_id = Uuid::parse_str(&payload.transaction_id).map_err(|e| {
        error!("Invalid transaction ID: {}", e);
        ApiError::Auth("Invalid transaction ID".to_string())
    })?;

    let capture_status = capture_result["status"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_string();

    match capture_status.as_str() {
        "COMPLETED" => {
            // Update transactions table
            let transaction = diesel::update(transactions::table)
                .filter(transactions::reference.eq(trans_id))
                .set((
                    transactions::status.eq("completed"),
                    transactions::description.eq("PayPal top-up completed"),
                ))
                .returning(Transaction::as_select())
                .get_result(conn)
                .map_err(|e| {
                    error!("Transaction update failed: {}", e);
                    if e.to_string().contains("not found") {
                        ApiError::Payment("Transaction not found".to_string())
                    } else {
                        ApiError::Database(e)
                    }
                })?;


            // Update wallets table
            diesel::insert_into(wallets::table)
                .values((
                    wallets::user_id.eq(transaction.user_id),
                    wallets::balance.eq(transaction.amount),
                    wallets::currency.eq("USD"),
                ))
                .on_conflict(wallets::user_id)
                .do_update()
                .set(wallets::balance.eq(wallets::balance + transaction.amount))
                .execute(conn)
                .map_err(|e| ApiError::Database(e))?;

            Ok(Json(CaptureResponse {
                status: "completed".to_string(),
                transaction_id: payload.transaction_id,
                error_message: None,
            }))
        }
        "DECLINED" | "FAILED" => {
            let error_message = capture_result["details"]
                .as_array()
                .and_then(|details| details.get(0))
                .and_then(|detail| detail["description"].as_str())
                .unwrap_or("Capture declined by PayPal")
                .to_string();

            diesel::update(transactions::table)
                .filter(transactions::reference.eq(trans_id))
                .set((
                    transactions::status.eq("failed"),
                    transactions::description
                        .eq(format!("PayPal top-up failed: {}", error_message)),
                ))
                .execute(conn)
                .map_err(|e| ApiError::Database(e))?;

            Ok(Json(CaptureResponse {
                status: "failed".to_string(),
                transaction_id: payload.transaction_id,
                error_message: Some(error_message),
            }))
        }
        "PENDING" => {
            let error_message = "Capture is pending, awaiting PayPal processing".to_string();
            diesel::update(transactions::table)
                .filter(transactions::reference.eq(trans_id))
                .set((
                    transactions::status.eq("pending"),
                    transactions::description.eq("PayPal top-up pending"),
                ))
                .execute(conn)
                .map_err(|e| ApiError::Database(e))?;

            Ok(Json(CaptureResponse {
                status: "pending".to_string(),
                transaction_id: payload.transaction_id,
                error_message: Some(error_message),
            }))
        }
        _ => {
            let error_message = format!("Unexpected capture status: {}", capture_status);
            diesel::update(transactions::table)
                .filter(transactions::reference.eq(trans_id))
                .set((
                    transactions::status.eq("failed"),
                    transactions::description
                        .eq(format!("PayPal top-up failed: {}", error_message)),
                ))
                .execute(conn)
                .map_err(|e| ApiError::Database(e))?;

            Err(ApiError::Payment(error_message).into())
        }
    }
}
