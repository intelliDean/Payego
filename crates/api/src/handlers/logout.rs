use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use payego_core::services::auth_service::logout::{
    LogoutService, Claims, ApiError, AppState, LogoutResponse
};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/logout",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Auth"
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
