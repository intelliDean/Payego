use axum::extract::State;
use axum::Extension;
use payego_primitives::config::security_config::Claims;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;


use axum::extract::Json;
use validator::Validate;
use payego_core::services::payment_service::{PaymentService, TopUpRequest, TopUpResponse};
use payego_primitives::error::ApiError;

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
    req.validate().map_err(ApiError::Validation)?;

    let user_id = claims.user_id()?;

    Ok(Json(
        PaymentService::initiate_top_up(&state, user_id, req).await?,
    ))
}



