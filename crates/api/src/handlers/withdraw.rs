use axum::extract::{Path, State};
use axum::{Extension, Json};
use payego_core::services::withdrawal_service::{
    ApiError, AppState, Claims, WithdrawRequest, WithdrawResponse, WithdrawalService,
};
use std::sync::Arc;
use tracing::log::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/wallets/withdraw/{bank_account_id}",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdrawal successful", body = WithdrawResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("bank_account_id" = Uuid, Path)
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
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    let res = WithdrawalService::withdraw(&state, user_id, bank_account_id, req).await?;

    Ok(Json(res))
}
