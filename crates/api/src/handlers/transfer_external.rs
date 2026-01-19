use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use payego_core::services::transfer_service::{
    ApiError, AppState, Claims, TransferRequest, TransferService,
};
use std::sync::Arc;
use tracing::log::error;
use validator::Validate;

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
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    TransferService::transfer_external(&state, user_id, req).await
}
