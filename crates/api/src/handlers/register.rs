use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, NewUser, RegisterRequest, RegisterResponse, User};
// Token generation now handled by JWTSecret::encode_token()
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Params
};
use diesel::prelude::*;
use payego_core::services::auth_service::AuthService;
use payego_primitives::config::security_config::create_token;
use std::sync::Arc;
use secrecy::{ExposeSecret, SecretBox, SecretString};
use tracing::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {

    payload.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let password = SecretString::new(payload.password.into());

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let password_hash = argon2id_hash_password(password)?;

    //create the user
    let new_user = NewUser {
        email: payload.email.clone(),
        password_hash,
        username: payload.username,
    };

    let user = diesel::insert_into(payego_primitives::schema::users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|e| {
            error!("User registration error: {}", e);
            ApiError::from(e)
        })?;

    let token = create_token(&state, &user.id.to_string()).map_err(|e| {
        error!("Token generation error: {}", e);
        ApiError::Internal("Failed to generate token".to_string())
    })?;

    let refresh_token = AuthService::generate_refresh_token(&mut conn, user.id).map_err(|e| {
        error!("Refresh token generation error: {}", e);
        ApiError::from(e)
    })?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            token,
            refresh_token,
            user_email: user.email,
        }),
    ))
}

fn argon2id_hash_password(password: SecretBox<str>) -> Result<String, ApiError> {
    //hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = create_argon2()?;
    let password_hash = argon2
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(|e| {
            error!("Argon2 hashing error: {}", e);
            ApiError::Internal("Encryption error".to_string())
        })?
        .to_string();
    Ok(password_hash)
}

pub fn create_argon2() -> Result<Argon2<'static>, ApiError> {

    let params = Params::new(
        65536, // 64 MiB memory
        3,     // iterations
        1,     // parallelism
        None,
    ).map_err(|e| {
        error!("Argon2 params error: {}", e);
        ApiError::Internal("Encryption configuration error".to_string())
    })?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    Ok(argon2)
}
