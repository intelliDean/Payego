use axum::extract::{Json, State};
use payego_core::services::paypal_service::{
    ApiError, AppState, CaptureRequest, CaptureResponse, PayPalService,
};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/paypal/capture",
    request_body = CaptureRequest,
    responses(
        (status = 200, body = CaptureResponse),
        (status = 409, description = "Already captured"),
        (status = 422, description = "Invalid payment state"),
        (status = 500)
    ),
    tag = "Payments"
)]
pub async fn paypal_capture(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CaptureRequest>,
) -> Result<Json<CaptureResponse>, ApiError> {
    let result = PayPalService::capture_order(&state, req.order_id, req.transaction_id).await?;

    Ok(Json(result))
}
