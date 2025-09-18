use crate::config::security_config::create_token;
use crate::error::ApiError;
use crate::models::user_models::{AppState, LoginRequest, LoginResponse, User};
use axum::{Json, extract::State, http::StatusCode};
use bcrypt::verify;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;
use validator::Validate;
use axum::http::header;

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
        tracing::error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    info!("Login attempt for email: {}", payload.email);

    // Get database connection with proper error handling
    let mut conn = state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Find user by email
    let user: Option<User> = crate::schema::users::table
        .filter(crate::schema::users::email.eq(&payload.email))
        .first(&mut conn)
        .optional()
        .map_err(ApiError::Database)?;

    let user = match user {
        Some(user) => user,
        None => {
            // Dummy verification to prevent timing attacks
            let _ = verify(
                &payload.password,
                "$2b$12$dummyhashdummyhashdummyhashdummyhashdummyhashdummyha",
            )
            .map_err(ApiError::Bcrypt)?;
            return Err(ApiError::Auth("Invalid email or password".to_string()).into());
        }
    };

    // Verify password
    if !verify(&payload.password, &user.password_hash).map_err(ApiError::Bcrypt)? {
        return Err(ApiError::Auth("Invalid email or password".to_string()).into());
    }

    // Generate JWT token with proper error handling
    let token = create_token(&state, &user.id.to_string())?;

    info!("User {} logged in successfully", user.id);



    // let response = Json(LoginResponse {
    //     token,
    //     user_email: user.email,
    // });
    //
    // (
    //     StatusCode::OK,
    //     [(
    //         header::SET_COOKIE,
    //         format!("jwt_token={}; HttpOnly; Path=/", token),
    //     )],
    //     response,
    // ))

    Ok(Json(LoginResponse {
        token,
        user_email: user.email,
    }))
}
