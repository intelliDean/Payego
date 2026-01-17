use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::transfer_service::TransferService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::dtos::WalletTransferRequest;

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
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let sender_id = claims.user_id()?;

    // Prevent self-transfer
    if sender_id == req.recipient_id {
        return Err(ApiError::Internal("Cannot transfer to yourself".into()));
    }

    let response = TransferService::transfer_internal(&state, sender_id, req).await?;

    Ok(response)
}

