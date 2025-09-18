use axum::Extension;
use axum::response::IntoResponse;
use crate::config::security_config::Claims;

#[utoipa::path(
    get,
    path = "/current_user",
    responses(
        (status = 200, description = "Get current user"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearerAuth" = [])),
    tag = "Auth"
)]
pub async fn get_current_user(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    format!("Current user: {}", claims.sub)
}
