use payego_primitives::error::ApiErrorResponse;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::transfer_service::{
    ApiError, AppState, Claims, TransferRequest, TransferService,
};
use std::sync::Arc;
use tracing::log::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/transfer/external",
    tag = "Wallet",
    summary = "Initiate external bank transfer from wallet",
    description = "Transfers funds from the authenticated user's wallet to an external bank account. \
                   Supports Nigerian banks (via Paystack Transfers API) or other providers depending on configuration. \
                   Requires sufficient wallet balance (including any applicable fees). \
                   The operation is **idempotent** when an `Idempotency-Key` header is provided — retries with the same key return the original initiation response without duplicate transfers. \
                   After initiation, the transfer status is usually `pending` and updated asynchronously via webhooks (`transfer.success`, `transfer.failed`, etc.). \
                   Always verify final status via webhook or polling before considering funds delivered.",
    operation_id = "initiateExternalTransfer",
    request_body(
        content = TransferRequest,
        description = "Transfer details: recipient account number, bank code, amount, currency, narration/description, \
                       and optional metadata (e.g. recipient name from resolve_account). \
                       Amount must be positive and within per-transaction / daily limits.",
    ),
    responses(
        ( status = 200, description = "Transfer successfully initiated (or idempotent retry). \
                           Returns transfer reference, status (usually 'pending'), and estimated delivery time."),
        ( status = 400, description = "Bad request — invalid input (insufficient balance, invalid bank code/account, amount out of limits, missing fields)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 402, description = "Payment required — insufficient wallet balance after fees", body = ApiErrorResponse),
        ( status = 409, description = "Conflict — duplicate transfer detected via idempotency key", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity — business rule violation (e.g. daily/monthly transfer limit exceeded, account not resolved)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on transfer initiations", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — failed to initiate transfer", body = ApiErrorResponse),
        ( status = 502, description = "Bad gateway — payment provider (Paystack) returned an error or is temporarily unavailable", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn transfer_external(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ApiError> {
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    TransferService::transfer_external(&state, user_id, req).await
}
