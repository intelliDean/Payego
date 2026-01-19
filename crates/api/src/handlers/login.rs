use axum::extract::{Json, State};
use std::sync::Arc;
use payego_core::services::auth_service::login::{
    LoginService, ApiError, AppState, LoginRequest, LoginResponse
};

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
    let response = LoginService::login(&state, payload).await?;
    Ok(Json(response))
}

