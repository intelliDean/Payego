use axum::body::Bytes;
use axum::{extract::State, http::StatusCode};
use payego_core::services::paystack_service::{
    ApiError, AppState, PaystackService, PaystackWebhook,
};
use std::sync::Arc;
// https://e02e3895d11f.ngrok-free.app/webhooks/paystack

#[utoipa::path(
    post,
    path = "/webhooks/paystack",
    request_body = PaystackWebhook,
    responses(
        (status = 200, description = "Webhook processed"),
        (status = 400, description = "Invalid webhook"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Webhook"
)]
pub async fn paystack_webhook(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    PaystackService::handle_event(state, headers, &body)?;

    Ok(StatusCode::OK)
}
