use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use axum::extract::{Json, Path, State};
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use payego_core::services::paypal_service::PayPalService;

#[derive(Debug, Deserialize)]
pub struct PayPalTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct PayPalOrderResponse {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub status: String,
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
) -> Result<Json<OrderResponse>, ApiError> {
    if order_id.len() < 10 {
        return Err(ApiError::Payment("Invalid PayPal order ID".into()));
    }

    let status = PayPalService::get_order_status(&state, &order_id).await?;

    Ok(Json(OrderResponse { status }))
}
