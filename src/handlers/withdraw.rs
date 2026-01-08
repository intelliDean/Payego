use axum::{
    extract::{State, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;
use utoipa::ToSchema;
use crate::{AppState, error::ApiError};
use crate::config::security_config::Claims;
use crate::services::withdrawal_service::WithdrawalService;

#[derive(Deserialize, ToSchema)]
pub struct WithdrawRequest {
    pub amount: f64, // Amount in the selected currency
    pub currency: String, // Currency to withdraw from (e.g., "USD", "NGN")
    pub bank_id: String, // Bank account ID from /api/bank_accounts
    pub reference: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct WithdrawResponse {
    pub transaction_id: String,
}

#[utoipa::path(
    post,
    path = "/api/withdraw",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdrawal initiated", body = WithdrawResponse),
        (status = 400, description = "Invalid amount, insufficient balance, or bank not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Transaction"
)]
pub async fn withdraw(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, (StatusCode, String)> {
    info!(
        "Withdrawal request: user_id={}, amount={}, currency={}, bank_id={}",
        claims.sub, req.amount, req.currency, req.bank_id
    );

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        info!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let response = WithdrawalService::initiate_withdrawal(state, user_id, req)
        .await
        .map_err(|e| {
             let (status, msg) = e.into();
             (status, msg)
        })?;

    Ok(Json(response))
}