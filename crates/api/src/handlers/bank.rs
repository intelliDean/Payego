use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::bank_account_service::{
    ApiError, AppState, BankAccount, BankAccountResponse, BankAccountService, BankRequest, Claims
};
use std::sync::Arc;
use tracing::error;
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
