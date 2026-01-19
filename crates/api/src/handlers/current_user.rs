use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_core::services::auth_service::token::TokenService;
use payego_core::services::auth_service::user::UserService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::token_dto::CurrentUserResponse;
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

    let response =
        UserService::current_user_summary(&state, claims.user_id()?).await?;

    Ok(Json(response))
}
