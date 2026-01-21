use crate::config::swagger_config::ApiErrorResponse;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::bank_account_service::{
    ApiError, AppState, BankAccount, BankAccountResponse, BankAccountService, BankRequest, Claims,
};
use std::sync::Arc;
use tracing::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/banks/add",
    tag = "Bank",
    summary = "Add or link a new bank account",
    description = "Adds a bank account to the authenticated user's profile. Use an `Idempotency-Key` header to make the operation safe against network flakes or duplicate submissions.",
    operation_id = "addBankAccount",
    request_body(content = BankRequest, description = "Details of the bank account to add or link"),
    responses(
        (status = 201, description = "Bank account added successfully", body = BankAccount),
        (status = 400,description = "Invalid input data (validation failed)",body = ApiErrorResponse),
        (status = 401,description = "Unauthorized – missing or invalid token",body = ApiErrorResponse),
        (status = 409,description = "Conflict – account already exists or duplicate request",body = ApiErrorResponse),
        (status = 429,description = "Too many requests (rate limit exceeded)",body = ApiErrorResponse),
        ( status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn add_bank_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<BankRequest>,
) -> Result<(StatusCode, Json<BankAccountResponse>), ApiError> {
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    let account = BankAccountService::create_bank_account(&state, user_id, req).await?;

    Ok((
        StatusCode::CREATED,
        Json(BankAccountResponse::from(account)),
    ))
}
