use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::transfer_service::TransferService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::{AppState, WalletTransferRequest};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/wallets/transfer",
    request_body = WalletTransferRequest,
    responses(
        (status = 200, description = "Transfer successful"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Wallet"
)]
pub async fn transfer_internal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<WalletTransferRequest>,
) -> Result<StatusCode, ApiError> {
    // 1. Validate request
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // 2. Parse user ID from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID in claims: {}", e);
        ApiError::Auth(AuthError::InvalidToken("Invalid user ID".to_string()))
    })?;

    // 3. Call TransferService
    let status = TransferService::transfer_internal(state, user_id, req).await?;

    Ok(status)
}
