use crate::error::ApiError;
use crate::models::models::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct OrderResponse {
    status: String,
}

#[utoipa::path(
    get,
    path = "/api/paypal/order/{order_id}",
    responses(
        (status = 200, description = "Order details retrieved", body = OrderResponse),
        (status = 400, description = "Invalid order ID"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Payments"
)]
pub async fn get_paypal_order(
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<String>,
) -> Result<Json<OrderResponse>, (StatusCode, String)> {
    let client = reqwest::Client::new();
    let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID")
        .map_err(|e| ApiError::Payment("PAYPAL_CLIENT_ID not set".to_string()))?;
    let paypal_secret = std::env::var("PAYPAL_SECRET")
        .map_err(|e| ApiError::Payment("PAYPAL_SECRET not set".to_string()))?;

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

    // Get order details
    let order_response = client
        .get(&format!(
            "https://api-m.sandbox.paypal.com/v2/checkout/orders/{}",
            order_id
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::Payment(format!("Failed to get PayPal order: {}", e)))?;

    let order_result: serde_json::Value = order_response
        .json()
        .await
        .map_err(|e| ApiError::Payment(format!("Failed to parse order response: {}", e)))?;

    let status = order_result["status"]
        .as_str()
        .ok_or(ApiError::Payment("Missing order status".to_string()))?
        .to_string();

    Ok(Json(OrderResponse { status }))
}
