use payego_primitives::error::ApiErrorResponse;
use axum::extract::{Json, State};
use payego_core::services::paypal_service::{
    ApiError, AppState, CaptureRequest, CaptureResponse, PayPalService,
};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/paypal/capture",
    tag = "Payments",
    summary = "Capture (finalize) a PayPal order payment",
    description = "Captures funds for a previously created and approved PayPal order. \
                   This transitions the order from `AUTHORIZED`/`APPROVED` to `COMPLETED`, deducting funds from the payer. \
                   The operation is **idempotent** when the `PayPal-Request-Id` (or your internal `Idempotency-Key`) header is provided — \
                   retries with the same key return the original result without duplicate charges. \
                   Use this after buyer approval (via redirect or advanced credit card flow). \
                   Supports partial captures if not final.",
    operation_id = "capturePaypalOrder",
    request_body(
        content = CaptureRequest,
        description = "Capture details: order ID, optional amount for partial capture, final_capture flag, \
                       tracking/shipping info, or payment source overrides. \
                       Body may be minimal/empty if the order already has an approved payment source.",
    ),
    responses(
        ( status = 200, description = "Capture successful (or idempotent retry) — funds captured, order completed. \
                           Returns capture details including transaction ID, amount breakdown, fees, and status.", body = CaptureResponse),
        ( status = 400, description = "Bad request — invalid or malformed input (e.g. missing fields, invalid order ID format)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — missing or invalid authentication (your JWT or PayPal access token issue)", body = ApiErrorResponse),
        ( status = 409, description = "Conflict — order already captured or in conflicting state (e.g. ORDER_ALREADY_CAPTURED)", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity — order in invalid state for capture (e.g. not approved/authorized, expired, insufficient funds, payer action required)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on capture attempts", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — capture failed due to server-side issue or PayPal service error", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn paypal_capture(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CaptureRequest>,
) -> Result<Json<CaptureResponse>, ApiError> {
    let result = PayPalService::capture_order(&state, req.order_id, req.transaction_id).await?;

    Ok(Json(result))
}
