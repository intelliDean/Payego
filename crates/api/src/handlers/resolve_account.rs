

use axum::{
    extract::{Query, State},
    Json,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;

use payego_core::services::bank_service::BankService;
use payego_primitives::{error::ApiError, models::app_state::app_state::AppState};

#[derive(Deserialize, ToSchema)]
pub struct ResolveAccountRequest {
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResolveAccountResponse {
    pub account_name: String,
}

static ACCOUNT_NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{10}$").unwrap());

static BANK_CODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{3,5}$").unwrap());

#[utoipa::path(
    get,
    path = "/api/resolve_account",
    params(
        ("bank_code" = String, Query, description = "Bank code (3-5 digits)"),
        ("account_number" = String, Query, description = "Account number (10 digits)")
    ),
    responses(
        (status = 200, description = "Account resolved", body = ResolveAccountResponse),
        (status = 400, description = "Invalid bank code or account number"),
        (status = 502, description = "Paystack API error")
    ),
    tag = "Verification"
)]
pub async fn resolve_account(
    State(state): State<Arc<AppState>>,
    Query(req): Query<ResolveAccountRequest>,
) -> Result<Json<ResolveAccountResponse>, ApiError> {

    if !ACCOUNT_NUMBER_RE.is_match(&req.account_number) {
        return Err(ApiError::Internal(
            "Account number must be 10 digits".to_string(),
        ));
    }

    if !BANK_CODE_RE.is_match(&req.bank_code) {
        return Err(ApiError::Internal(
            "Bank code must be 3–5 digits".to_string(),
        ));
    }

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