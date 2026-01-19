use axum::{extract::Json, extract::State, Extension};
use payego_core::services::payment_service::{
    PaymentService, ApiError, AppState, TopUpRequest, TopUpResponse, Claims
};
use std::sync::Arc;
use tracing::log::error;
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
    
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    Ok(Json(
        PaymentService::initiate_top_up(&state, user_id, req).await?,
    ))
}
