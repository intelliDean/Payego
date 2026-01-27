use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use payego_core::services::auth_service::logout::{
    ApiError, AppState, Claims, LogoutResponse, LogoutService,
};
use payego_primitives::error::ApiErrorResponse;
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Authentication",
    summary = "Log out the current user",
    description = "Invalidates the current JWT access token (and optionally any associated refresh tokens or sessions). \
                   After successful logout, the client should discard the token locally and no longer use it for authenticated requests. \
                   This endpoint requires a valid token — calling it with an already expired or invalid token may return 401. \
                   This is a best-effort operation; the server may not always be able to revoke tokens immediately in distributed systems.",
    operation_id = "logoutUser",
    responses(
        ( status = 200, description = "Logout successful — current session/token has been invalidated", body = LogoutResponse),
        ( status = 401, description = "Unauthorized — missing, invalid, expired, or already revoked token", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on logout attempts", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — logout could not be processed", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<LogoutResponse>), ApiError> {
    LogoutService::logout(&state, claims).await?;

    Ok((
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Logged out successfully".into(),
        }),
    ))
}
