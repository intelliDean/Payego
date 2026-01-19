use axum::extract::{Json, State};
use diesel::prelude::*;
use payego_core::services::paypal_service::PayPalService;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::providers_dto::{CaptureRequest, CaptureResponse};
use std::sync::Arc;
use utoipa::ToSchema;

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
    let result =
        PayPalService::capture_order(&state, req.order_id, req.transaction_id).await?;

    Ok(Json(result))
}
