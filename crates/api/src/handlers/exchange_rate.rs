use axum::{
    extract::{Query, State},
    Json,
};
use payego_core::services::conversion_service::{ApiError, AppState, ConversionService};
use payego_primitives::models::enum_types::CurrencyCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use payego_primitives::error::ApiErrorResponse;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExchangeRateQuery {
    pub from: CurrencyCode,
    pub to: CurrencyCode,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExchangeRateResponse {
    pub from: String,
    pub to: String,
    pub rate: f64,
}

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
        (status = 200, description = "Exchange rate retrieved successfully", body = ExchangeRateResponse),
        (status = 400, description = "Bad request - invalid currency codes", body = ApiErrorResponse),
        (status = 500, description = "Internal server error - failed to fetch exchange rate", body = ApiErrorResponse),
    ),
)]
pub async fn get_exchange_rate(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExchangeRateQuery>,
) -> Result<Json<ExchangeRateResponse>, ApiError> {
    let rate = ConversionService::get_exchange_rate(&state, params.from, params.to).await?;

    Ok(Json(ExchangeRateResponse {
        from: params.from.to_string(),
        to: params.to.to_string(),
        rate,
    }))
}
