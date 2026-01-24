use payego_primitives::error::ApiErrorResponse;
use axum::extract::{Json, Path, State};
use payego_core::services::paypal_service::{ApiError, AppState, OrderResponse, PayPalService};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/paypal/order/{order_id}",
    tag = "Payments",
    summary = "Retrieve PayPal order details",
    description = "Fetches the current status and full details of a PayPal order by its ID. \
                   Useful for checking payment approval status, amount, payer information, \
                   shipping details, and whether the order is ready for capture. \
                   Returns the latest state from PayPal (e.g. CREATED, APPROVED, COMPLETED, VOIDED). \
                   This endpoint requires authentication and the order must belong to the authenticated user.",
    operation_id = "getPaypalOrderDetails",
    responses(
        ( status = 200, description = "Successfully retrieved PayPal order details. Returns current order state, amount, payer info, etc.", body = OrderResponse),
        ( status = 400, description = "Bad request – invalid or malformed order ID format", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized – missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden – order does not belong to the authenticated user or insufficient permissions", body = ApiErrorResponse),
        ( status = 404, description = "Not found – order ID does not exist or was not created through this API", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests – rate limit exceeded for order lookups", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error – failed to retrieve order from PayPal or internal issue", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = []))
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
