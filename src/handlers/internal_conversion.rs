use axum::{
    extract::{State, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;
use validator::Validate;
use regex::Regex;
use tracing::{error, info};
use utoipa::ToSchema;
use crate::{AppState, error::ApiError};
use crate::config::security_config::Claims;
use crate::services::conversion_service::ConversionService;

static SUPPORTED_CURRENCIES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(USD|NGN|GBP|EUR|CAD|AUD|JPY|CHF|CNY|SEK|NZD|MXN|SGD|HKD|NOK|KRW|TRY|INR|BRL|ZAR)$",
    )
        .expect("Invalid currency")
});

#[derive(Deserialize, ToSchema, Validate)]
pub struct ConvertRequest {
    #[validate(range(min = 1.0, max = 10000.0, message = "Amount must be between 1 and 10,000"))]
    pub amount: f64,
    #[validate(regex(path = "SUPPORTED_CURRENCIES", message = "Invalid from currency"))]
    pub from_currency: String,
    #[validate(regex(path = "SUPPORTED_CURRENCIES", message = "Invalid to currency"))]
    pub to_currency: String,
}

#[derive(Serialize, ToSchema)]
pub struct ConvertResponse {
    pub transaction_id: String,
    pub converted_amount: f64,
    pub exchange_rate: f64,
    pub fee: f64,
}


#[utoipa::path(
    post,
    path = "/api/convert_currency",
    request_body = ConvertRequest,
    responses(
        (status = 200, description = "Currency converted successfully", body = ConvertResponse),
        (status = 400, description = "Invalid input or insufficient balance"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Currency"
)]
pub async fn convert_currency(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, (StatusCode, String)> {
    info!("Convert currency initiated");

    // Validate input
    req.validate().map_err(|e: validator::ValidationErrors| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let response = ConversionService::convert_currency(state, user_id, req)
        .await
        .map_err(|e| {
             let (status, msg) = e.into();
             (status, msg)
        })?;

    Ok(Json(response))
}
