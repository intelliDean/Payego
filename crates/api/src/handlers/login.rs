use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, LoginRequest, LoginResponse, User};
// Token generation now handled by JWTSecret::encode_token()
use payego_primitives::config::security_config::create_token;
use payego_core::services::auth_service::AuthService;
use axum::extract::{Json, State};
use bcrypt::verify;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::error;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let mut conn = state
        .db
        .get()
        .map_err(|e| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

    let user = payego_primitives::schema::users::table
        .filter(payego_primitives::schema::users::email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| {
            error!("Database error during login: {}", e);
            ApiError::from(e)
        })?
        .ok_or_else(|| {
            error!("User not found: {}", payload.email);
            ApiError::Auth("Invalid credentials".to_string())
        })?;

    if !verify(&payload.password, &user.password_hash).map_err(|e| {
        error!("Bcrypt verification error: {}", e);
        ApiError::Internal("Encryption error".to_string())
    })? {
        return Err(ApiError::Auth("Invalid credentials".to_string()));
    }

    let token = create_token(&state, &user.id.to_string()).map_err(|e| {
        error!("Token generation error: {}", e);
        ApiError::Internal("Failed to generate token".to_string())
    })?;

    let refresh_token = AuthService::generate_refresh_token(&mut conn, user.id).map_err(|e| {
        error!("Refresh token generation error: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(LoginResponse {
        token,
        refresh_token,
        user_email: user.email,
    }))
}
