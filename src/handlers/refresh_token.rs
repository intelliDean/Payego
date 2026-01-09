use crate::config::security_config::create_token;
use crate::error::ApiError;
use crate::models::models::{AppState, LoginResponse};
use crate::services::auth_service::AuthService;
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
pub struct RefreshRequest {
    pub user_id: Uuid,
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
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // Validate request
    payload.validate().map_err(|e| {
        tracing::error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let mut conn = state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Validate and rotate refresh token
    let new_refresh_token = AuthService::validate_and_rotate_refresh_token(
        &mut conn,
        payload.user_id,
        &payload.refresh_token,
    )?;

    // Generate new access token
    let access_token = create_token(&state, &payload.user_id.to_string())?;

    // Get user details for response (optional, but LoginResponse expects email)
    let user_email: String = crate::schema::users::table
        .filter(crate::schema::users::id.eq(payload.user_id))
        .select(crate::schema::users::email)
        .first::<String>(&mut conn)
        .map_err(ApiError::Database)?;

    Ok(Json(LoginResponse {
        token: access_token,
        refresh_token: new_refresh_token,
        user_email,
    }))
}
