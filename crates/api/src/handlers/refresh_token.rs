use axum::{extract::State, Json};
use payego_core::services::auth_service::token::{
    ApiError, AppState, LoginResponse, RefreshRequest, SecurityConfig, TokenService,
};
use std::sync::Arc;
use tracing::log::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = LoginResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    payload.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Validate refresh token and rotate it
    let refreshed =
        TokenService::validate_and_rotate_refresh_token(&state, &payload.refresh_token)?;

    // refreshed should contain: user_id, new_refresh_token, user_email
    let access_token = SecurityConfig::create_token(&state, &refreshed.user_id.to_string())?;

    Ok(Json(LoginResponse {
        token: access_token,
        refresh_token: refreshed.new_refresh_token,
        user_email: None,
    }))
}
