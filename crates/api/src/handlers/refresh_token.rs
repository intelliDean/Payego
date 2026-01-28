use axum::{extract::State, Json};
use payego_core::services::auth_service::token::{
    ApiError, AppState, RefreshRequest, RefreshResponse, SecurityConfig, TokenService,
};
use payego_primitives::error::ApiErrorResponse;
use std::sync::Arc;
use tracing::warn;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Authentication",
    summary = "Refresh access token using refresh token",
    description = "Exchanges a valid refresh token for a new access token (and optionally a new refresh token). \
                   Used to extend user sessions without requiring re-login. \
                   The old refresh token may be invalidated (rotation pattern) or remain valid depending on your security policy. \
                   This is a **public endpoint** — no bearer access token is required, only the refresh token in the body. \
                   Refresh tokens typically have longer lifetimes than access tokens but shorter than session cookies.",
    operation_id = "refreshToken",
    request_body(
        content = RefreshRequest,
        description = "Payload containing the current refresh token. \
                       May also include device identifier or client metadata for enhanced security in some implementations.",
    ),
    responses(
        ( status = 200, description = "Token refreshed successfully — returns new access token (and possibly new refresh token)", body = RefreshResponse),
        ( status = 400, description = "Bad request — invalid or malformed refresh token format", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — refresh token is invalid, expired, revoked, or already used (in rotation mode)", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden — refresh token was revoked (e.g. user logged out from all devices, suspicious activity detected)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on refresh attempts", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — token refresh could not be processed", body = ApiErrorResponse),
    ),
    security(()),
)]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ApiError> {
    payload.validate().map_err(|e| {
        warn!("refresh_token: validation error");
        ApiError::Validation(e)
    })?;

    // Validate refresh token and rotate it
    let refreshed =
        TokenService::validate_and_rotate_refresh_token(&state, &payload.refresh_token)?;

    let access_token = SecurityConfig::create_token(&state, &refreshed.user_id.to_string())?;

    Ok(Json(RefreshResponse {
        token: access_token,
        refresh_token: refreshed.new_refresh_token,
    }))
}
