use crate::config::swagger_config::ApiErrorResponse;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use payego_core::services::auth_service::register::{
    ApiError, AppState, RegisterService, {RegisterRequest, RegisterResponse},
};
use std::sync::Arc;
use tracing::log::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Authentication",
    summary = "Register a new user account",
    description = "Creates a new user account with the provided credentials and profile information. \
                   On success, returns the created user details and (depending on your flow) an access token or verification instructions. \
                   This is a **public endpoint** — no authentication is required. \
                   Email uniqueness is enforced. Passwords are hashed securely on the server. \
                   Depending on configuration, the user may need to verify their email before full access is granted.",
    operation_id = "registerUser",
    request_body(
        content = RegisterRequest,
        description = "User registration details: email, password, name, and optional fields (phone, referral code, etc.). \
                       Password must meet minimum complexity requirements (length, characters, etc.).",
    ),
    responses(
        ( status = 201, description = "User successfully registered. Returns user profile data and possibly initial access/refresh tokens \
                           or a message indicating next steps (e.g. email verification required).", body = RegisterResponse),
        ( status = 400, description = "Bad request — invalid or missing input fields (e.g. invalid email format, weak password, missing required fields)", body = ApiErrorResponse),
        ( status = 409, description = "Conflict — an account with this email already exists", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — registration rate limit exceeded (prevents abuse/spam)", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — registration could not be completed", body = ApiErrorResponse),
    ),
    security(()),
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
    payload.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let response = RegisterService::register(&state, payload).await?;

    Ok((StatusCode::CREATED, Json(response)))
}
