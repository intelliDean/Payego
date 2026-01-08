use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use regex::Regex;
use lazy_static::lazy_static;
use tracing::info;
use utoipa::ToSchema;
use crate::AppState;
use crate::services::bank_service::BankService;

#[derive(Deserialize, ToSchema)]
pub struct ResolveAccountRequest {
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResolveAccountResponse {
    pub account_name: String,
}

lazy_static! {
    static ref ACCOUNT_NUMBER_RE: Regex = Regex::new(r"^\d{10}$").expect("Invalid account number regex");
}

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
    State(_state): State<Arc<AppState>>,
    Query(req): Query<ResolveAccountRequest>,
) -> Result<Json<ResolveAccountResponse>, (StatusCode, String)> {
    info!("Resolve account initiated for bank_code: {}, account_number: {}", req.bank_code, req.account_number);

    // Validate input
    if !ACCOUNT_NUMBER_RE.is_match(&req.account_number) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Account number must be 10 digits".to_string(),
        ));
    }

    let account_name = BankService::resolve_account_details(&req.bank_code, &req.account_number)
        .await
        .map_err(|e| {
             let (status, msg) = e.into();
             (status, msg)
        })?;

    info!("Account resolved: {} - {}", req.account_number, account_name);

    Ok(Json(ResolveAccountResponse { account_name }))
}