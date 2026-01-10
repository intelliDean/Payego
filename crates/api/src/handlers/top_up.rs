use axum::extract::{Extension, Json, State};
use payego_core::services::payment_service::PaymentService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, TopUpRequest, TopUpResponse};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/top_up",
    request_body = TopUpRequest,
    responses(
        (status = 200, description = "Payment initiated", body = TopUpResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Transaction"
)]
pub async fn top_up(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TopUpRequest>,
) -> Result<Json<TopUpResponse>, ApiError> {
    // 1. Validate request
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // 2. Parse user ID from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID in claims: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // 3. Call PaymentService
    let response = PaymentService::initiate_top_up(&state, user_id, req).await?;

    Ok(Json(response))
}
