use axum::{
    extract::{Query, State},
    Json,
};
use payego_core::services::bank_account_service::{
    ApiError, AppState, BankAccountService, ResolveAccountRequest, ResolveAccountResponse,
};
use payego_primitives::error::ApiErrorResponse;
use std::sync::Arc;
use tracing::info;

#[utoipa::path(
    get,
    path = "/api/bank/resolve",
    tag = "Bank",
    summary = "Resolve bank account name",
    description = "Verifies a bank account number and retrieves the account holder's name using the Paystack Resolve Account API. \
                   This helps confirm the correct recipient before initiating transfers or payments. \
                   Requires valid Nigerian bank code (from Paystack supported banks list) and 10-digit account number. \
                   The endpoint is rate-limited and depends on Paystack availability — cache results when possible for repeated lookups.",
    operation_id = "resolveAccountName",
    params(
        ("account_number" = String, Query, description = "10-digit bank account number to verify"),
        ("bank_code" = String, Query, description = "Bank code from Paystack supported banks list (e.g., '058' for GTBank)")
    ),
    responses(
        ( status = 200, description = "Account successfully resolved — returns account name and other verification details", body = ResolveAccountResponse),
        ( status = 400, description = "Bad request — invalid bank code, account number format, or missing required parameters", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 404, description = "Not found — bank code not supported or account does not exist", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded (per user or globally)", body = ApiErrorResponse),
        ( status = 502, description = "Bad gateway — Paystack API is unavailable or returned an unexpected error", body = ApiErrorResponse),
        ( status = 503, description = "Service unavailable — Paystack account resolution temporarily down or maintenance", body = ApiErrorResponse),
    ),
    security(()),
)]
pub async fn resolve_account(
    State(state): State<Arc<AppState>>,
    Query(req): Query<ResolveAccountRequest>,
) -> Result<Json<ResolveAccountResponse>, ApiError> {
    info!(
        "Resolving account ****{} @ {}",
        &req.account_number[6..],
        req.bank_code
    );

    let resolved =
        BankAccountService::resolve_account_details(&state, &req.bank_code, &req.account_number)
            .await?;

    Ok(Json(ResolveAccountResponse {
        account_name: resolved.account_name,
    }))
}
