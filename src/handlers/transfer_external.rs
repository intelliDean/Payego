use crate::config::security_config::Claims;
use crate::models::models::{PayoutRequest};
use crate::{AppState, error::ApiError};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use std::sync::{Arc, LazyLock};
use regex::Regex;
use tracing::{error, info};
use uuid::Uuid;

// Static regex for account number validation
static ACCOUNT_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{10}$").expect("Invalid account number regex")
});

#[utoipa::path(
    post,
    path = "/api/transfer/external",
    request_body = PayoutRequest,
    responses(
        (status = 200, description = "Payout initiated"),
        (status = 400, description = "Invalid bank or insufficient balance")
    ),
    security(("bearerAuth" = []))
)]
pub async fn external_transfer(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PayoutRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!(
        "External transfer request: sender={}, recipient_account_number={}, amount={}, currency={}",
        claims.sub, req.account_number, req.amount, req.currency
    );

    // Validate input (account_number)    
    if !ACCOUNT_NUMBER_RE.is_match(&req.account_number) {
        error!("Invalid account number: {}", req.account_number);
        return Err(ApiError::Payment("Account number must be 10 digits".to_string()).into());
    }

    // Validate amount
    if req.amount <= 0.0 {
        error!("Invalid amount: {}", req.amount);
        return Err(ApiError::Payment("Amount must be positive".to_string()).into());
    }

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Delegate to Service
    crate::services::transfer_service::TransferService::execute_external_transfer(
        state,
        user_id,
        req,
    )
    .await
    .map_err(|e| e.into())
}
