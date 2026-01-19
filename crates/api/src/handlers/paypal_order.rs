use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use axum::extract::{Json, Path, State};
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use payego_core::services::paypal_service::PayPalService;
use payego_primitives::models::providers_dto::OrderResponse;

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
