use axum::extract::{Path, State};
use axum::{Extension, Json};
use payego_core::services::withdrawal_service::{
    ApiError, AppState, Claims, WithdrawRequest, WithdrawResponse, WithdrawalService,
};
use payego_primitives::error::ApiErrorResponse;
use std::sync::Arc;
use tracing::log::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/wallet/withdraw/{bank_account_id}",
    tag = "Wallet",
    summary = "Initiate withdrawal from wallet to bank account",
    description = "Withdraws funds from the authenticated user's wallet to a previously linked and verified bank account. \
                   The specified `bank_account_id` must belong to the current user and be verified. \
                   The operation is **idempotent** when an `Idempotency-Key` header is provided — \
                   retries with the same key return the original initiation response without duplicate withdrawals. \
                   Withdrawal amount must be positive, within available balance (after fees), \
                   and respect per-transaction / daily / monthly limits. \
                   Most withdrawals are asynchronous: status starts as `pending` and updates via webhooks (`transfer.success`, `transfer.failed`, etc.). \
                   Always rely on final webhook confirmation — do **not** assume success from the 200 response alone.",
    operation_id = "initiateWalletWithdrawal",
    request_body(
        content = WithdrawRequest,
        description = "Withdrawal details: amount, currency (must match wallet), optional narration/description, \
                       and any provider-specific options. \
                       Example: `{ \"amount\": 4500.00, \"currency\": \"NGN\", \"narration\": \"Personal withdrawal\" }`",
    ),
    responses(
        ( status = 200, description = "Withdrawal successfully initiated (or idempotent retry). \
                           Returns withdrawal reference, pending status, estimated delivery time, \
                           and transaction ID for tracking.", body = WithdrawResponse),
        ( status = 400, description = "Bad request — invalid input (amount ≤ 0, unsupported currency, missing fields, invalid narration length)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 402, description = "Payment required — insufficient available balance (after fees)", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden — bank account not verified, not owned by user, or withdrawal not allowed (e.g. frozen wallet)", body = ApiErrorResponse),
        ( status = 404, description = "Not found — bank account ID does not exist or does not belong to the user", body = ApiErrorResponse),
        ( status = 409, description = "Conflict — duplicate withdrawal attempt detected via idempotency key", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity — business rule violation (daily/monthly limit exceeded, minimum withdrawal amount not met)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on withdrawal initiations", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — failed to initiate withdrawal", body = ApiErrorResponse),
        ( status = 502, description = "Bad gateway — payment provider (Paystack) failed to accept the withdrawal request", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn withdraw(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(bank_account_id): Path<Uuid>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, ApiError> {
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    let res = WithdrawalService::withdraw(&state, user_id, bank_account_id, req).await?;

    Ok(Json(res))
}
