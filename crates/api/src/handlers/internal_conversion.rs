use axum::extract::{Extension, Json, State};
use payego_core::services::conversion_service::ConversionService;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::conversion_dto::{ConvertRequest, ConvertResponse};
use std::sync::Arc;
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

    req.validate()?;


    let response =
        ConversionService::convert_currency(&state, claims.user_id()?, req).await?;

    Ok(Json(response))
}
