use crate::config::swagger_config::ApiErrorResponse;
use axum::{
    extract::{Query, State},
    Json,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use tracing::info;

use payego_core::services::bank_service::{
    ApiError, AppState, BankService, ResolveAccountRequest, ResolveAccountResponse,
};

static ACCOUNT_NUMBER_RE: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| Regex::new(r"^\d{10}$"));

static BANK_CODE_RE: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| Regex::new(r"^\d{3,5}$"));
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
    validate_account_number(&req)?;

    info!(
        "Resolving account ****{} @ {}",
        &req.account_number[6..],
        req.bank_code
    );

    let resolved =
        BankService::resolve_account_details(&state, &req.bank_code, &req.account_number).await?;

    Ok(Json(ResolveAccountResponse {
        account_name: resolved.account_name,
    }))
}

fn validate_account_number(req: &ResolveAccountRequest) -> Result<(), ApiError> {
    if !ACCOUNT_NUMBER_RE
        .as_ref()
        .map_err(|_| ApiError::Internal("Account number regex misconfigured".into()))?
        .is_match(&req.account_number)
    {
        return Err(ApiError::Internal(
            "Account number must be 10 digits".to_string(),
        ));
    }

    if !BANK_CODE_RE
        .as_ref()
        .map_err(|_| ApiError::Internal("Account number regex misconfigured".into()))?
        .is_match(&req.bank_code)
    {
        return Err(ApiError::Internal(
            "Bank code must be 3–5 digits".to_string(),
        ));
    }

    Ok(())
}
