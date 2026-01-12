use axum::extract::{Extension, Json, Path, State};
use payego_core::services::withdrawal_service::WithdrawalService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::{AppState, WithdrawRequest, WithdrawResponse};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/wallets/withdraw/{bank_account_id}",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdrawal initiated", body = WithdrawResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("bank_account_id" = Uuid, Path, description = "Bank account ID to withdraw to")
    ),
    security(("bearerAuth" = [])),
    tag = "Wallet"
)]
pub async fn withdraw(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(bank_account_id): Path<Uuid>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, ApiError> {
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

    // 3. Call WithdrawalService
    let response = WithdrawalService::withdraw(&*state, user_id, bank_account_id, req).await?;

    Ok(Json(response))
}
