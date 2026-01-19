use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::transfer_service::{TransferService};
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use validator::Validate;
use payego_primitives::models::transfer_dto::TransferRequest;

#[utoipa::path(
    post,
    path = "/api/wallets/transfer_external",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer initiated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Wallet"
)]
pub async fn transfer_external(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, ApiError> {
    req.validate()?;
    let user_id = claims.user_id()?;

    TransferService::transfer_external(&state, user_id, req).await
}
