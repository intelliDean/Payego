use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use regex::Regex;
use lazy_static::lazy_static;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use crate::{AppState, error::ApiError};

#[derive(Deserialize, ToSchema)]
pub struct ResolveAccountRequest {
    bank_code: String,
    account_number: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResolveAccountResponse {
    account_name: String,
}

lazy_static! {
    // static ref BANK_CODE_RE: Regex = Regex::new(r"^\d{3,5}$").expect("Invalid bank code regex");
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
    State(state): State<Arc<AppState>>,
    Query(req): Query<ResolveAccountRequest>,
) -> Result<Json<ResolveAccountResponse>, (StatusCode, String)> {
    info!("Resolve account initiated for bank_code: {}, account_number: {}", req.bank_code, req.account_number);

    // Validate input
    if !ACCOUNT_NUMBER_RE.is_match(&req.account_number) {
        error!("Invalid account number: {}", req.account_number);
        return Err((
            StatusCode::BAD_REQUEST,
            "Account number must be 10 digits".to_string(),
        ));
    }

    let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
        error!("PAYSTACK_SECRET_KEY not set");
        ApiError::Payment("Paystack configuration error".to_string())
    })?;

    let client = Client::new();
    let url = format!(
        "https://api.paystack.co/bank/resolve?account_number={}&bank_code={}",
        req.account_number, req.bank_code
    );
    debug!("Sending Paystack request: {}", url);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", paystack_key))
        .send()
        .await
        .map_err(|e| {
            error!("Paystack resolve API error: {}", e);
            ApiError::Payment(format!("Failed to connect to Paystack: {}", e))
        })?;

    let status = resp.status();
    let body = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            error!("Paystack response parsing error: {}", e);
            ApiError::Payment(format!("Invalid Paystack response: {}", e))
        })?;

    debug!("Paystack response: {:?}", body);

    if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
        let message = body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack error")
            .to_string();
        error!("Paystack account resolution failed: {}", message);
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Account resolution failed: {}", message),
        ));
    }

    let account_name = body["data"]["account_name"]
        .as_str()
        .ok_or_else(|| {
            error!("Missing account_name in Paystack response: {:?}", body);
            ApiError::Payment("Invalid Paystack response: missing account_name".to_string())
        })?
        .to_string();

    info!("Account resolved: {} - {}", req.account_number, account_name);

    Ok(Json(ResolveAccountResponse { account_name }))
}