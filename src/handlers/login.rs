// use crate::config::security_config::create_token;
// use crate::error::ApiError;
// use crate::models::models::{AppState, LoginRequest, LoginResponse, User};
// use axum::{Json, extract::State, http::StatusCode};
// use bcrypt::verify;
// use diesel::prelude::*;
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use tracing::info;
// use utoipa::ToSchema;
// use validator::Validate;
// use axum::http::header;
//
// #[utoipa::path(
//     post,
//     path = "/api/login",
//     request_body = LoginRequest,
//     responses(
//         (status = 200, description = "Login successful", body = LoginResponse),
//         (status = 400, description = "Invalid input"),
//         (status = 401, description = "Invalid email or password"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Auth"
// )]
// pub async fn login(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<LoginRequest>,
// ) -> Result<Json<LoginResponse>, (StatusCode, String)> {
//     // Validate input
//     payload.validate().map_err(|e| {
//         tracing::error!("Validation error: {}", e);
//         ApiError::Validation(e)
//     })?;
//
//     info!("Login attempt for email: {}", payload.email);
//
//     // Get database connection with proper error handling
//     let mut conn = state.db.get().map_err(|e| {
//         tracing::error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     // Find user by email
//     let user: Option<User> = crate::schema::users::table
//         .filter(crate::schema::users::email.eq(&payload.email))
//         .first(&mut conn)
//         .optional()
//         .map_err(ApiError::Database)?;
//
//     let user = match user {
//         Some(user) => user,
//         None => {
//             // Dummy verification to prevent timing attacks
//             let _ = verify(
//                 &payload.password,
//                 "$2b$12$dummyhashdummyhashdummyhashdummyhashdummyhashdummyha",
//             )
//             .map_err(ApiError::Bcrypt)?;
//             return Err(ApiError::Auth("Invalid email or password".to_string()).into());
//         }
//     };
//
//     // Verify password
//     if !verify(&payload.password, &user.password_hash).map_err(ApiError::Bcrypt)? {
//         return Err(ApiError::Auth("Invalid email or password".to_string()).into());
//     }
//
//     // Generate JWT token with proper error handling
//     let token = create_token(&state, &user.id.to_string())?;
//
//     info!("User {} logged in successfully", user.id);
//
//
//     Ok(Json(LoginResponse {
//         token,
//         user_email: user.email,
//     }))
// }



use crate::config::security_config::{create_token, is_token_blacklisted};
use crate::error::ApiError;
use crate::models::models::{AppState, LoginRequest, LoginResponse, User};
use axum::{Json, extract::State, http::StatusCode};
use bcrypt::verify;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Invalid email or password"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // Validate input
    payload.validate().map_err(|e| {
        tracing::error!("Validation error for email {}: {}", payload.email, e);
        ApiError::Validation(e)
    })?;

    info!("Login attempt for email: {}", payload.email);

    // Get database connection
    let mut conn = state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Find user by email
    let user: Option<User> = crate::schema::users::table
        .filter(crate::schema::users::email.eq(&payload.email))
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            tracing::error!("Database error finding user {}: {}", payload.email, e);
            ApiError::Database(e)
        })?;

    let user = match user {
        Some(user) => user,
        None => {
            // Dummy verification to prevent timing attacks
            let _ = verify(
                &payload.password,
                "$2b$12$dummyhashdummyhashdummyhashdummyhashdummyhashdummyha",
            )
                .map_err(ApiError::Bcrypt)?;
            tracing::warn!("No user found for email: {}", payload.email);
            return Err(ApiError::Auth("Invalid email or password".to_string()).into());
        }
    };

    // Verify password
    if !verify(&payload.password, &user.password_hash).map_err(|e| {
        tracing::error!("Password verification error for user {}: {}", user.id, e);
        ApiError::Bcrypt(e)
    })? {
        tracing::warn!("Invalid password for user: {}", user.id);
        return Err(ApiError::Auth("Invalid email or password".to_string()).into());
    }

    // Generate JWT token
    let mut token = create_token(&state, &user.id.to_string())?;
    let mut attempts = 0;

    // Ensure token is not blacklisted (rare case)
    while is_token_blacklisted(&mut conn, &token)? && attempts < 3 {
        tracing::warn!("Generated token is blacklisted, retrying for user {} (attempt {})", user.id, attempts + 1);
        token = create_token(&state, &user.id.to_string())?;
        attempts += 1;
    }

    if attempts >= 3 {
        tracing::error!("Failed to generate non-blacklisted token for user {}", user.id);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate valid token".to_string(),
        ));
    }

    info!("User {} logged in successfully with token ending in ...{}", user.id, token.chars().rev().take(8).collect::<String>());

    Ok(Json(LoginResponse {
        token,
        user_email: user.email,
    }))
}
