use axum::{
    extract::{Query, State},
    Json,
};
use payego_core::services::conversion_service::{ApiError, AppState, ConversionService};
use payego_primitives::error::ApiErrorResponse;
use payego_primitives::models::enum_types::CurrencyCode;
use payego_primitives::models::{ExchangeQuery, ExchangeResponse};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/exchange-rate",
    tag = "Exchange",
    summary = "Get current exchange rate between two currencies",
    description = "Returns the current exchange rate for converting from one currency to another. \
                   This is a public endpoint that doesn't require authentication.",
    params(
        ("from" = CurrencyCode, Query, description = "Source currency code"),
        ("to" = CurrencyCode, Query, description = "Target currency code"),
    ),
    responses(
        (status = 200, description = "Exchange rate retrieved successfully", body = ExchangeResponse),
        (status = 400, description = "Bad request - invalid currency codes", body = ApiErrorResponse),
        (status = 500, description = "Internal server error - failed to fetch exchange rate", body = ApiErrorResponse),
    ),
)]
pub async fn get_exchange_rate(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExchangeQuery>,
) -> Result<Json<ExchangeResponse>, ApiError> {
    let rate = ConversionService::get_exchange_rate(&state, params.from, params.to).await?;

    Ok(Json(ExchangeResponse {
        from: params.from.to_string(),
        to: params.to.to_string(),
        rate,
    }))
}
