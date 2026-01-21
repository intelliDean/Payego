use crate::models::enum_types::CurrencyCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct WithdrawRequest {
    #[validate(range(min = 0.01))]
    pub amount: f64,

    pub currency: CurrencyCode,

    pub reference: Uuid,

    #[validate(length(min = 10, max = 128))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WithdrawResponse {
    pub transaction_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletSummaryDto {
    pub currency: CurrencyCode,
    pub balance: i64,
}
