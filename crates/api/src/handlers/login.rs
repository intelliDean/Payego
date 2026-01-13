use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::{AppState, LoginRequest, LoginResponse, User};
// Token generation now handled by JWTSecret::encode_token()
use axum::extract::{Json, State};
use argon2::{password_hash::{PasswordHash, PasswordVerifier}, Argon2, Params};
use diesel::prelude::*;
use payego_core::services::auth_service::AuthService;
use payego_primitives::config::security_config::create_token;
use std::sync::Arc;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use tracing::error;
use crate::handlers::register::create_argon2;

#[utoipa::path(
    post,
    path = "/api/login",
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
    use payego_primitives::schema::users::dsl::*;

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection error during login: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch user if exists
    let user_opt = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| {
            error!("Database error during login lookup: {}", e);
            ApiError::Database(e)
        })?;

    // Always parse a hash to avoid timing attacks
    let password_hsh = match &user_opt {
        Some(user) => user.password_hash.as_str(),
        None => "$argon2id$v=19$m=65536,t=3,p=1$invalidsalt$invalidhash",
    };

    let parsed_hash = PasswordHash::new(password_hsh).map_err(|e| {
        error!("Password hash parsing failure: {}", e);
        ApiError::Internal("Authentication failure".into())
    })?;
    
    let argon2 = create_argon2()?;

    let password_valid = argon2
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    let user = match (user_opt, password_valid) {
        (Some(user), true) => user,
        _ => return Err(ApiError::Auth(AuthError::InvalidCredentials)),
    };

    let token = create_token(&state, &user.id.to_string()).map_err(|e| {
        error!("JWT generation failed: {}", e);
        ApiError::Internal("Authentication service error".into())
    })?;

    let refresh_token =
        AuthService::generate_refresh_token(&mut conn, user.id).map_err(|e| {
            error!("Refresh token generation failed: {}", e);
            ApiError::Internal("Authentication service error".into())
        })?;

    Ok(Json(LoginResponse {
        token,
        refresh_token,
        user_email: Some(user.email),
    }))
}
