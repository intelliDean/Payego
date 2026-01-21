use crate::models::enum_types::{CurrencyCode, PaymentProvider};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TopUpRequest {
    #[validate(range(min = 1.0, max = 10_000.0))]
    pub amount: f64,

    pub provider: PaymentProvider,
    #[schema(example = "NGN")]
    pub currency: CurrencyCode,

    #[validate(length(min = 8, max = 128))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopUpResponse {
    pub session_url: Option<String>,
    pub payment_id: Option<String>,
    pub transaction_id: String,
    pub amount: f64,
}
