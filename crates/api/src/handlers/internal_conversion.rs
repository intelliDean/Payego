use crate::config::swagger_config::ApiErrorResponse;
use axum::extract::{Extension, Json, State};
use payego_core::services::conversion_service::{
    ApiError, AppState, Claims, ConversionService, ConvertRequest, ConvertResponse,
};
use std::sync::Arc;
use tracing::error;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/wallets/convert",
    tag = "Wallet",
    summary = "Convert currency within user's wallet",
    description = "Converts an amount from one currency to another within the authenticated user's wallet. \
                   Uses real-time (or cached) exchange rates. The operation is **idempotent** when an `Idempotency-Key` header is provided. \
                   Fees (if any) are deducted from the source amount before conversion. \
                   The source wallet must have sufficient balance.",
    operation_id = "convertCurrency",
    request_body(content = ConvertRequest, description = "Details of the conversion: source currency/amount, target currency, optional idempotency key usage",),
    responses(
        ( status = 200, description = "Conversion completed successfully. Returns updated wallet balances and transaction reference.", body = ConvertResponse),
        ( status = 400, description = "Bad request – invalid input (e.g. unsupported currencies, insufficient balance, invalid amount, same source/target currency)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized – missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 409, description = "Conflict – duplicate request detected via idempotency key (returns original successful response if previously completed)", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity – validation failed (e.g. amount ≤ 0, unsupported currency pair, rate fetch failed)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests – rate limit exceeded (e.g. too many conversions in short time)", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error – conversion could not be completed (e.g. exchange rate service unavailable)", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = []))
)]
pub async fn convert_currency(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    let response = ConversionService::convert_currency(&state, user_id, req).await?;

    Ok(Json(response))
}
