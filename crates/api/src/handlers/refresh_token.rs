use axum::{extract::State, Json};
use payego_primitives::config::security_config::{ SecurityConfig};
use payego_primitives::error::ApiError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use payego_core::services::auth_service::token::TokenService;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::login_dto::LoginResponse;

#[derive(Deserialize, ToSchema, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 64))]
    pub refresh_token: String,
}


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
    
    payload.validate().map_err(ApiError::Validation)?;
    
    // Validate refresh token and rotate it
    let refreshed = TokenService::validate_and_rotate_refresh_token(
        &state,
        &payload.refresh_token,
    )?;

    // refreshed should contain: user_id, new_refresh_token, user_email
    let access_token = SecurityConfig::create_token(&state, &refreshed.user_id.to_string())?;

    Ok(Json(LoginResponse {
        token: access_token,
        refresh_token: refreshed.new_refresh_token,
        user_email: None,
    }))
}
