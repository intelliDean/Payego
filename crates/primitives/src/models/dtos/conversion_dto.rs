use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use crate::models::enum_types::CurrencyCode;

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