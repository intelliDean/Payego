use axum::extract::{Path, State};
use axum::{Extension, Json};
use payego_core::services::withdrawal_service::WithdrawalService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::withdrawal_dto::{WithdrawRequest, WithdrawResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    req.validate()?;

    let user_id = claims.user_id()?;

    let res = WithdrawalService::withdraw(state.as_ref(), user_id, bank_account_id, req).await?;

    Ok(Json(res))
}
