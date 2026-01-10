use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, BankAccount, BankRequest};
use payego_core::services::bank_service::BankService;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/bank_accounts",
    request_body = BankRequest,
    responses(
        (status = 201, description = "Bank account added successfully", body = BankAccount),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Bank Account"
)]
pub async fn add_bank_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<BankRequest>,
) -> Result<(StatusCode, Json<BankAccount>), ApiError> {
    // 1. Validate request
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // 2. Parse user ID from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID in claims: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // 3. Call BankService
    let account = BankService::add_bank_account(&state, user_id, req)
        .await?;

    Ok((StatusCode::CREATED, Json(account)))
}
