use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use payego_core::services::auth_service::register::{
    ApiError, AppState, RegisterService, {RegisterRequest, RegisterResponse},
};
use std::sync::Arc;
use tracing::log::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
    payload.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let response = RegisterService::register(&state, payload).await?;

    Ok((StatusCode::CREATED, Json(response)))
}
