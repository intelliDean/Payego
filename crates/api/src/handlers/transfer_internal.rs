use axum::extract::{Extension, Json, State};
use payego_core::services::transfer_service::{
    ApiError, AppState, Claims, TransferService, WalletTransferRequest,
};
use payego_primitives::error::ApiErrorResponse;
use std::sync::Arc;
use tracing::warn;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/transfer/internal",
    tag = "Wallet",
    summary = "Transfer funds between user wallets (internal)",
    description = "Initiates an internal transfer of funds from the authenticated user's wallet to another user's wallet (by email, phone, username, or wallet ID). \
                   This is a **real-time** operation — funds are debited and credited instantly upon success (no pending state in most cases). \
                   The operation is **idempotent** when an `Idempotency-Key` header is provided — retries with the same key return the original result without double-spending. \
                   Requires sufficient balance in the source wallet (including any internal transfer fee, if applicable). \
                   Supports optional memo/description for record-keeping. \
                   Both sender and recipient must have active wallets in the system.",
    operation_id = "transferInternalWallet",
    request_body(
        content = WalletTransferRequest,
        description = "Internal transfer details: recipient identifier (email/phone/username/wallet ID), amount, currency, optional memo.",
    ),
    responses(
        (
            status = 200,
            description = "Transfer completed successfully (or idempotent retry). \
                           Funds are immediately debited from sender and credited to recipient wallet."),
        ( status = 400, description = "Bad request — invalid input (negative amount, unsupported currency, missing recipient, invalid identifier format)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 402, description = "Payment required — insufficient wallet balance (after fees)", body = ApiErrorResponse),
        ( status = 404, description = "Not found — recipient wallet not found (invalid email/phone/username/wallet ID)", body = ApiErrorResponse),
        ( status = 409, description = "Conflict — duplicate transfer detected via idempotency key", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity — business rule violation (e.g. daily transfer limit exceeded, self-transfer not allowed)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on transfer attempts", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — failed to process internal transfer", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn transfer_internal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    payload: Result<Json<WalletTransferRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let Json(req) = payload
        .map_err(|_rejection| {
            warn!("transfer_internal: invalid JSON payload");
            ApiError::Validation(validator::ValidationErrors::new())
        })
        .map_err(|_| ApiError::Internal("Invalid JSON payload".into()))?;

    req.validate().map_err(|e| {
        warn!("transfer_internal: validation error");
        ApiError::Validation(e)
    })?;

    let sender_id = claims.user_id()?;

    // Prevent self-transfer
    if sender_id == req.recipient {
        return Err(ApiError::Internal("Cannot transfer to yourself".into()));
    }

    // let recipient_id = req.recipient;

    let transaction_id = TransferService::transfer_internal(&state, sender_id, req).await?;

    Ok(Json(
        serde_json::json!({ "id": transaction_id.to_string() }),
    ))
}
