use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::bank_service::BankService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::bank::BankAccount;
use payego_primitives::models::bank_dtos::BankRequest;

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

    // 3. Call BankService
    let account = BankService::create_bank_account(
        &state,
        claims.user_id()?, 
        req
    ).await?;

    Ok((StatusCode::CREATED, Json(account)))
}