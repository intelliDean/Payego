use crate::models::enum_types::CurrencyCode;
use diesel::Queryable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TransferRequest {
    #[validate(range(min = 1.0, max = 10000.0))]
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
    pub recipient: Uuid,
    #[validate(range(min = 1.0, max = 10000.0))]
    pub amount: f64,
    pub currency: CurrencyCode,
    pub description: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Deserialize)]
pub struct ResolveUserRequest {
    pub identifier: String,
}

#[derive(Serialize, Queryable, ToSchema)]
pub struct ResolvedUser {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
}
