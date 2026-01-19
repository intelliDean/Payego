use axum::{
    extract::{Extension, State},
    Json,
};
use payego_core::services::auth_service::user::{
    ApiError, AppState, CurrentUserResponse, UserService, Claims
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/current_user",
    responses(
        (status = 200, description = "User data retrieved successfully", body = CurrentUserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "User"
)]
pub async fn current_user_details(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<CurrentUserResponse>, ApiError> {
    let user_id = claims.user_id()?;

    let response = UserService::current_user_summary(&state, user_id).await?;

    Ok(Json(response))
}
