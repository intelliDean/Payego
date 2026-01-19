use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use payego_primitives::config::security_config::{Claims};
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::app_state::app_state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use payego_core::services::auth_service::logout::LogoutService;
use payego_primitives::models::token_dto::LogoutResponse;

#[utoipa::path(
    post,
    path = "/api/logout",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<LogoutResponse>), ApiError> {
    LogoutService::logout(&state, claims).await?;

    Ok((
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Logged out successfully".into(),
        }),
    ))
}