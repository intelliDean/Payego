use crate::config::swagger_config::ApiErrorResponse;
use axum::extract::{Json, State};
use payego_core::services::auth_service::login::{
    ApiError, AppState, LoginRequest, LoginResponse, LoginService,
};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Authentication",
    summary = "Authenticate user and obtain JWT token",
    description = "Authenticates a user using email and password\
                   On success, returns a JWT access token, email and a refresh token that can be used \
                   for subsequent authenticated requests via the `Authorization: Bearer <token>` header. \
                   This is a public endpoint — no prior authentication is required.",
    operation_id = "loginUser",
    request_body(
        content = LoginRequest,
        description = "User credentials for authentication (email/username + password). \
                       Depending on your configuration, may also support OAuth-like flows or 2FA codes in future.",
    ),
    responses(
        ( status = 200, description = "Login successful — returns JWT access token and user basic info", body = LoginResponse),
        ( status = 400, description = "Bad request — invalid or missing input fields (e.g. malformed email, empty password)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — invalid credentials (wrong email/password combination)", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden — account is locked, suspended, or requires email verification / 2FA", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded (login attempt throttling)", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — authentication could not be processed", body = ApiErrorResponse),
    ),
    security(()),
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let response = LoginService::login(&state, payload).await?;
    Ok(Json(response))
}
