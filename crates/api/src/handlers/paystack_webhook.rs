use payego_primitives::error::ApiErrorResponse;
use axum::body::Bytes;
use axum::{extract::State, http::StatusCode};
use payego_core::services::paystack_service::{
    ApiError, AppState, PaystackService, PaystackWebhook,
};
use std::sync::Arc;
// https://e02e3895d11f.ngrok-free.app/webhooks/paystack

#[utoipa::path(
    post,
    path = "/api/webhooks/paystack",
    tag = "Webhooks",
    summary = "Receive and process Paystack webhook events",
    description = "Public endpoint that receives asynchronous event notifications from Paystack (e.g. charge.success, transfer.success, subscription.create, etc.). \
                   The server **must** verify the request signature using the `X-Paystack-Signature` header and your secret key before processing. \
                   Always respond with HTTP 200 OK as quickly as possible — even if processing fails internally — to avoid Paystack retrying the event. \
                   This is a **public endpoint** (no bearer token required). Duplicate events are possible; your handler should be **idempotent**. \
                   Events are sent as JSON with an `event` field and `data` payload.",
    operation_id = "receivePaystackWebhook",
    request_body(
        content = PaystackWebhook,
        description = "Paystack webhook payload containing the event type and data.",
    ),
    responses(
        ( status = 200, description = "Webhook received and acknowledged. Paystack expects 200 OK — do not return error status even if processing fails later.",),
        ( status = 400, description = "Bad request — invalid payload, failed signature verification, malformed JSON, or unrecognized event type", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity — valid signature but event data invalid or unexpected (e.g. missing required fields)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded (protects your server from flood)", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — should be avoided. Paystack will retry on 5xx responses.", body = ApiErrorResponse),
    ),
    security(()),
)]
pub async fn paystack_webhook(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    PaystackService::handle_event(state, headers, &body)?;

    Ok(StatusCode::OK)
}
