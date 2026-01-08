use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::models::models::AppState;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use std::sync::Arc;
use tracing::{error};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};
use serde::{Deserialize, Serialize};

const SUPPORTED_PROVIDERS: &[&str] = &["stripe", "paypal"];
const SUPPORTED_CURRENCIES: &[&str] = &[
    "USD", "EUR", "GBP", "AUD", "BRL", "CAD", "CHF", "CNY", "HKD", "INR", "JPY", "KRW", "MXN",
    "NGN", "NOK", "NZD", "SEK", "SGD", "TRY", "ZAR",
];

#[derive(Deserialize, Validate, ToSchema)]
pub struct TopUpRequest {
    #[validate(range(
        min = 1.0,
        max = 10000.0,
        message = "Amount must be between 1 and 10,000"
    ))]
    pub amount: f64,
    #[validate(custom(function = "validate_provider"))]
    pub provider: String,
    #[validate(custom(function = "validate_currency"))]
    pub currency: String,
    pub reference: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub idempotency_key: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TopUpResponse {
    pub session_url: Option<String>,
    pub payment_id: Option<String>,
    pub transaction_id: String,
    pub amount: f64,
}

fn validate_provider(provider: &str) -> Result<(), ValidationError> {
    if SUPPORTED_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err(ValidationError::new("Provider must be 'stripe' or 'paypal'"))
    }
}

fn validate_currency(currency: &str) -> Result<(), ValidationError> {
    if SUPPORTED_CURRENCIES.contains(&currency) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid currency"))
    }
}

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
) -> Result<Json<TopUpResponse>, (StatusCode, String)> {
    // Validate request
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Delegate to service
    let response = crate::services::payment_service::PaymentService::initiate_top_up(
        state.clone(),
        user_id,
        req,
    ).await?;

    Ok(Json(response))
}
