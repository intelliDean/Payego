use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use payego_core::services::withdrawal_service::WithdrawalService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::dtos::{WithdrawRequest, WithdrawResponse};

#[utoipa::path(
    post,
    path = "/api/wallets/withdraw/{bank_account_id}",
    request_body = WithdrawRequest,
    responses(
        (status = 200, body = WithdrawResponse),
        (status = 400),
        (status = 401),
        (status = 500)
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

    let res = WithdrawalService::withdraw(
        state.as_ref(),
        user_id,
        bank_account_id,
        req,
    )
        .await?;

    Ok(Json(res))
}
