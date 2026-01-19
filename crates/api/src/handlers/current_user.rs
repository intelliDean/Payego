use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_core::services::auth_service::{token::TokenService};
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use payego_core::services::auth_service::user::UserService;
use payego_primitives::models::token_dto::CurrentUserResponse;

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
    let user_id = claims
        .sub
        .parse()
        .map_err(|_| ApiError::Auth(AuthError::InvalidToken("Invalid subject".into())))?;

    let response = UserService::current_user_summary(&state, user_id).await?;
    Ok(Json(response))
}