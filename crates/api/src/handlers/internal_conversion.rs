use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, ConvertRequest, ConvertResponse};
use payego_core::services::conversion_service::ConversionService;
use axum::extract::{Extension, Json, State};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/wallets/convert",
    request_body = ConvertRequest,
    responses(
        (status = 200, description = "Conversion successful", body = ConvertResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Wallet"
)]
pub async fn convert_currency(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
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

    // 3. Call ConversionService
    let response = ConversionService::convert_currency(&*state, user_id, req)
        .await?;

    Ok(Json(response))
}
