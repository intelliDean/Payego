use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// --- BANK DTOS ---
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct BankRequest {
    pub bank_name: String,
    pub account_number: String,
    pub bank_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankResponse {
    pub id: Uuid,
    pub bank_name: String,
    pub account_number: String,
    pub account_name: String,
}
