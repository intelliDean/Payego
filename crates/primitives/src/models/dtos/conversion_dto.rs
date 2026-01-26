use crate::models::enum_types::CurrencyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ConvertRequest {
    #[validate(range(min = 1))]
    pub amount_cents: i64,
    pub from_currency: CurrencyCode,
    pub to_currency: CurrencyCode,
    #[validate(length(min = 8, max = 128))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConvertResponse {
    pub transaction_id: String,
    pub converted_amount: f64,
    pub exchange_rate: f64,
    pub fee: f64,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRateResponse {
    pub rates: HashMap<String, f64>,
    pub error: Option<String>,
}


pub struct ConvertQuoteRequest {
    pub from_currency: CurrencyCode,
    pub to_currency: CurrencyCode,
    pub amount_cents: i64,
}

pub struct ConvertQuoteResponse {
    pub quote_id: Uuid,
    pub exchange_rate: f64,
    pub fee: f64,
    pub net_amount: f64,
    pub expires_at: DateTime<Utc>,
}

pub struct ConfirmConvertRequest {
    pub quote_id: Uuid,
    pub idempotency_key: String,
}

