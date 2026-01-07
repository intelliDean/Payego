// use crate::AppState;
// use crate::error::ApiError;
// use crate::schema::{transactions, wallets};
// use axum::{
//     extract::{Json, State},
//     http::{HeaderMap, StatusCode},
// };
// use diesel::prelude::*;
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use tracing::{error, info};
// use utoipa::ToSchema;
// use uuid::Uuid;
//
//
// #[derive(Queryable, Selectable, Debug)]
// #[diesel(table_name = transactions)]
// pub struct Transaction {
//     pub reference: uuid::Uuid,
//     pub user_id: uuid::Uuid,
//     pub amount: i64,
//     pub currency: String,
// }
//
// #[derive(Deserialize, ToSchema)]
// pub struct CaptureRequest {
//     pub order_id: String,
//     pub transaction_id: String,
// }
//
// #[derive(Serialize, ToSchema)]
// pub struct CaptureResponse {
//     pub status: String,
//     pub transaction_id: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub error_message: Option<String>,
// }
//
// #[utoipa::path(
//     post,
//     path = "/api/paypal/capture",
//     request_body = CaptureRequest,
//     responses(
//         (status = 200, description = "Order captured successfully", body = CaptureResponse),
//         (status = 400, description = "Invalid order ID or transaction ID"),
//         (status = 401, description = "Unauthorized"),
//         (status = 500, description = "Internal server error")
//     ),
//     security(("bearerAuth" = [])),
//     tag = "Payments"
// )]
// pub async fn paypal_capture(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<CaptureRequest>,
// ) -> Result<Json<CaptureResponse>, (StatusCode, String)> {
//     let client = Client::new();
//     let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID")
//         .map_err(|e| {
//             error!("PayPal client ID not set: {}", e);
//             ApiError::Payment("PAYPAL_CLIENT_ID not set".to_string())
//         })?;
//     let paypal_secret = std::env::var("PAYPAL_SECRET")
//         .map_err(|e| {
//             error!("PayPal secret not set: {}", e);
//             ApiError::Payment("PAYPAL_SECRET not set".to_string())
//         })?;
//
//     info!(
//         "PayPal Capture initiation with ORDER ID: {}, TRANSACTION ID: {}",
//         &payload.order_id, &payload.transaction_id
//     );
//
//     // Get PayPal access token
//     let token_response = client
//         .post("https://api-m.sandbox.paypal.com/v1/oauth2/token")
//         .basic_auth(&paypal_client_id, Some(&paypal_secret))
//         .form(&[("grant_type", "client_credentials")])
//         .send()
//         .await
//         .map_err(|e| {
//             error!("Failed to get PayPal token: {}", e);
//             ApiError::Payment(format!("Failed to get PayPal token: {}", e))
//         })?;
//
//     info!("Token response status: {}", token_response.status());
//
//     let token: serde_json::Value = token_response
//         .json()
//         .await
//         .map_err(|e| {
//             error!("Failed to parse PayPal token: {}", e);
//             ApiError::Payment(format!("Failed to parse PayPal token: {}", e))
//         })?;
//     let access_token = token["access_token"]
//         .as_str()
//         .ok_or_else(|| {
//             error!("Missing access_token in PayPal response");
//             ApiError::Payment("Missing access_token".to_string())
//         })?;
//
//     // Capture the order
//     let capture_response = client
//         .post(&format!(
//             "https://api-m.sandbox.paypal.com/v2/checkout/orders/{}/capture",
//             payload.order_id
//         ))
//         .bearer_auth(access_token)
//         .header("Content-Type", "application/json")
//         .send()
//         .await
//         .map_err(|e| {
//             error!("PayPal capture failed: {}", e);
//             ApiError::Payment(format!("PayPal capture failed: {}", e))
//         })?;
//
//     info!("Capture response status: {}", capture_response.status());
//
//     let capture_result: serde_json::Value = capture_response
//         .json()
//         .await
//         .map_err(|e| {
//             error!("Failed to parse capture response: {}", e);
//             ApiError::Payment(format!("Failed to parse capture response: {}", e))
//         })?;
//
//     info!("Capture result: {}", capture_result);
//
//     let conn = &mut state
//         .db
//         .get()
//         .map_err(|e| {
//             error!("Database connection error: {}", e);
//             ApiError::DatabaseConnection(e.to_string())
//         })?;
//
//     let trans_id = Uuid::parse_str(&payload.transaction_id).map_err(|e| {
//         error!("Invalid transaction ID: {}", e);
//         ApiError::Auth("Invalid transaction ID".to_string())
//     })?;
//
//     // Fetch transaction to validate currency and get details
//     let transaction = transactions::table
//         .filter(transactions::reference.eq(trans_id))
//         .select(Transaction::as_select())
//         .first(conn)
//         .map_err(|e| {
//             error!("Failed to fetch transaction: {}", e);
//             ApiError::Payment("Transaction not found".to_string())
//         })?;
//
//     // Validate currency
//     let paypal_currency = capture_result["purchase_units"][0]["payments"]["captures"][0]["amount"]["currency_code"]
//         .as_str()
//         .ok_or_else(|| {
//             error!("Missing currency in PayPal capture response");
//             ApiError::Payment("Missing currency in capture response".to_string())
//         })?
//         .to_uppercase();
//     info!("transaction currency: {}", transaction.currency);
//     info!("paypal currency: {}", paypal_currency);
//
//     if transaction.currency != paypal_currency {
//         error!(
//             "Currency mismatch: transaction currency {}, PayPal currency {}",
//             transaction.currency, paypal_currency
//         );
//         return Err((
//             StatusCode::BAD_REQUEST,
//             "Currency mismatch".to_string(),
//         ));
//     }
//
//     let capture_status = capture_result["status"]
//         .as_str()
//         .unwrap_or("UNKNOWN")
//         .to_string();
//
//     match capture_status.as_str() {
//         "COMPLETED" => {
//             info!("Initiating Complete Transaction");
//             // Update transactions and wallets atomically
//             conn.transaction(|conn| {
//                 // Update transactions table
//                 let updated_transaction = diesel::update(transactions::table)
//                     .filter(transactions::reference.eq(trans_id))
//                     .set((
//                         transactions::status.eq("completed"),
//                         transactions::description.eq("PayPal top-up completed"),
//                     ))
//                     .returning(Transaction::as_select())
//                     .get_result(conn)
//                     .map_err(|e| {
//                         error!("Transaction update failed: {}", e);
//                         ApiError::Payment("Transaction not found".to_string())
//                     })?;
//
//                 // Update wallets table
//                 diesel::insert_into(wallets::table)
//                     .values((
//                         wallets::user_id.eq(updated_transaction.user_id),
//                         wallets::balance.eq(updated_transaction.amount),
//                         wallets::currency.eq(updated_transaction.currency),
//                     ))
//                     .on_conflict((wallets::user_id, wallets::currency)) // Fixed to match schema
//                     .do_update()
//                     .set(wallets::balance.eq(wallets::balance + updated_transaction.amount))
//                     .execute(conn)
//                     .map_err(|e| {
//                         error!("Wallet update failed: {}", e);
//                         ApiError::Database(e)
//                     })?;
//
//                 Ok::<(), ApiError>(())
//             })?;
//
//             Ok(Json(CaptureResponse {
//                 status: "completed".to_string(),
//                 transaction_id: payload.transaction_id,
//                 error_message: None,
//             }))
//         }
//         "DECLINED" | "FAILED" => {
//             let error_message = capture_result["details"]
//                 .as_array()
//                 .and_then(|details| details.get(0))
//                 .and_then(|detail| detail["description"].as_str())
//                 .unwrap_or("Capture declined by PayPal")
//                 .to_string();
//
//             diesel::update(transactions::table)
//                 .filter(transactions::reference.eq(trans_id))
//                 .set((
//                     transactions::status.eq("failed"),
//                     transactions::description.eq(format!("PayPal top-up failed: {}", error_message)),
//                 ))
//                 .execute(conn)
//                 .map_err(|e| {
//                     error!("Transaction update failed: {}", e);
//                     ApiError::Database(e)
//                 })?;
//
//             Ok(Json(CaptureResponse {
//                 status: "failed".to_string(),
//                 transaction_id: payload.transaction_id,
//                 error_message: Some(error_message),
//             }))
//         }
//         "PENDING" => {
//             let error_message = "Capture is pending, awaiting PayPal processing".to_string();
//             diesel::update(transactions::table)
//                 .filter(transactions::reference.eq(trans_id))
//                 .set((
//                     transactions::status.eq("pending"),
//                     transactions::description.eq("PayPal top-up pending"),
//                 ))
//                 .execute(conn)
//                 .map_err(|e| {
//                     error!("Transaction update failed: {}", e);
//                     ApiError::Database(e)
//                 })?;
//
//             Ok(Json(CaptureResponse {
//                 status: "pending".to_string(),
//                 transaction_id: payload.transaction_id,
//                 error_message: Some(error_message),
//             }))
//         }
//         _ => {
//             let error_message = format!("Unexpected capture status: {}", capture_status);
//             diesel::update(transactions::table)
//                 .filter(transactions::reference.eq(trans_id))
//                 .set((
//                     transactions::status.eq("failed"),
//                     transactions::description.eq(format!("PayPal top-up failed: {}", error_message)),
//                 ))
//                 .execute(conn)
//                 .map_err(|e| {
//                     error!("Transaction update failed: {}", e);
//                     ApiError::Database(e)
//                 })?;
//
//             Err(ApiError::Payment(error_message).into())
//         }
//     }
// }

//=================



use crate::AppState;
use crate::error::ApiError;
use crate::schema::{transactions, wallets};
use axum::{
    extract::{Json, State},
    http::{StatusCode},
};
use diesel::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub reference: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub amount: i64,
    pub currency: String,
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
        (status = 422, description = "PayPal payment declined"),
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
    let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|e| {
        error!("PayPal client ID not set: {}", e);
        ApiError::Payment("PAYPAL_CLIENT_ID not set".to_string())
    })?;
    let paypal_secret = std::env::var("PAYPAL_SECRET").map_err(|e| {
        error!("PayPal secret not set: {}", e);
        ApiError::Payment("PAYPAL_SECRET not set".to_string())
    })?;

    info!(
        "PayPal Capture initiation with ORDER ID: {}, TRANSACTION ID: {}",
        &payload.order_id, &payload.transaction_id
    );

    // Get PayPal access token
    let token_response = client
        .post("https://api-m.sandbox.paypal.com/v1/oauth2/token")
        .basic_auth(&paypal_client_id, Some(&paypal_secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to get PayPal token: {}", e);
            ApiError::Payment(format!("Failed to get PayPal token: {}", e))
        })?;

    let token: serde_json::Value = token_response
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse PayPal token: {}", e);
            ApiError::Payment(format!("Failed to parse PayPal token: {}", e))
        })?;
    let access_token = token["access_token"]
        .as_str()
        .ok_or_else(|| {
            error!("Missing access_token in PayPal response");
            ApiError::Payment("Missing access_token".to_string())
        })?;

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
        .map_err(|e| {
            error!("PayPal capture failed: {}", e);
            ApiError::Payment(format!("PayPal capture failed: {}", e))
        })?;

    let status = capture_response.status();
    let capture_result: serde_json::Value = capture_response
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse capture response: {}", e);
            ApiError::Payment(format!("Failed to parse capture response: {}", e))
        })?;

    info!("Capture response status: {}, result: {}", status, capture_result);

    if !status.is_success() {
        let error_message = capture_result["details"]
            .as_array()
            .and_then(|details| details.get(0))
            .and_then(|detail| detail["description"].as_str())
            .unwrap_or(capture_result["message"].as_str().unwrap_or("Unknown PayPal error"))
            .to_string();
        error!("PayPal capture failed with status {}: {}", status, error_message);

        let conn = &mut state
            .db
            .get()
            .map_err(|e| {
                error!("Database connection error: {}", e);
                ApiError::DatabaseConnection(e.to_string())
            })?;

        let trans_id = Uuid::parse_str(&payload.transaction_id).map_err(|e| {
            error!("Invalid transaction ID: {}", e);
            ApiError::Payment("Invalid transaction ID".to_string())
        })?;

        // Update transaction to failed
        diesel::update(transactions::table)
            .filter(transactions::reference.eq(trans_id))
            .set((
                transactions::status.eq("failed"),
                transactions::description.eq(format!("PayPal top-up failed: {}", error_message)),
            ))
            .execute(conn)
            .map_err(|e| {
                error!("Transaction update failed: {}", e);
                ApiError::Database(e)
            })?;

        return Ok(Json(CaptureResponse {
            status: "failed".to_string(),
            transaction_id: payload.transaction_id,
            error_message: Some(error_message),
        }));
    }

    let conn = &mut state
        .db
        .get()
        .map_err(|e| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

    let trans_id = Uuid::parse_str(&payload.transaction_id).map_err(|e| {
        error!("Invalid transaction ID: {}", e);
        ApiError::Payment("Invalid transaction ID".to_string())
    })?;

    // Fetch transaction to validate currency and get details
    let transaction = transactions::table
        .filter(transactions::reference.eq(trans_id))
        .select(Transaction::as_select())
        .first(conn)
        .optional()
        .map_err(|e| {
            error!("Failed to fetch transaction: {}", e);
            ApiError::Database(e)
        })?
        .ok_or_else(|| {
            error!("Transaction not found: {}", trans_id);
            ApiError::Payment("Transaction not found".to_string())
        })?;

    // Validate currency
    let paypal_currency = capture_result["purchase_units"][0]["payments"]["captures"][0]["amount"]["currency_code"]
        .as_str()
        .map(|s| s.to_uppercase())
        .unwrap_or_default();
    if paypal_currency.is_empty() || transaction.currency != paypal_currency {
        error!(
            "Currency mismatch: transaction currency {}, PayPal currency {}",
            transaction.currency, paypal_currency
        );
        return Err(ApiError::Payment(format!(
            "Currency mismatch: expected {}, got {}",
            transaction.currency, paypal_currency
        ))
            .into());
    }

    let capture_status = capture_result["status"]
        .as_str()
        .unwrap_or("UNKNOWN")
        .to_string();

    match capture_status.as_str() {
        "COMPLETED" => {
            info!("Initiating Complete Transaction for {}", trans_id);
            // Update transactions and wallets atomically
            conn.transaction(|conn| {
                // Update transactions table
                let updated_transaction = diesel::update(transactions::table)
                    .filter(transactions::reference.eq(trans_id))
                    .set((
                        transactions::status.eq("completed"),
                        transactions::description.eq("PayPal top-up completed"),
                    ))
                    .returning(Transaction::as_select())
                    .get_result(conn)
                    .map_err(|e| {
                        error!("Transaction update failed: {}", e);
                        ApiError::Database(e)
                    })?;

                // Update wallets table
                diesel::insert_into(wallets::table)
                    .values((
                        wallets::user_id.eq(updated_transaction.user_id),
                        wallets::balance.eq(updated_transaction.amount),
                        wallets::currency.eq(updated_transaction.currency),
                    ))
                    .on_conflict((wallets::user_id, wallets::currency))
                    .do_update()
                    .set(wallets::balance.eq(wallets::balance + updated_transaction.amount))
                    .execute(conn)
                    .map_err(|e| {
                        error!("Wallet update failed: {}", e);
                        ApiError::Database(e)
                    })?;

                Ok::<(), ApiError>(())
            })
                .map_err(|e| e)?;

            info!("PayPal payment captured for transaction: {}", trans_id);
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
                    transactions::description.eq(format!("PayPal top-up failed: {}", error_message)),
                ))
                .execute(conn)
                .map_err(|e| {
                    error!("Transaction update failed: {}", e);
                    ApiError::Database(e)
                })?;

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
                .map_err(|e| {
                    error!("Transaction update failed: {}", e);
                    ApiError::Database(e)
                })?;

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
                    transactions::description.eq(format!("PayPal top-up failed: {}", error_message)),
                ))
                .execute(conn)
                .map_err(|e| {
                    error!("Transaction update failed: {}", e);
                    ApiError::Database(e)
                })?;

            Err(ApiError::Payment(error_message).into())
        }
    }
}
