use crate::models::enum_types::CurrencyCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TransferRequest {
    pub amount: f64,
    pub currency: String,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct WalletTransferRequest {
    pub recipient_id: Uuid,
    pub amount: f64,
    pub currency: CurrencyCode,
    pub description: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}
