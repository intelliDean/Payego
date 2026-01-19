use crate::models::enum_types::CurrencyCode;
use serde::{Deserialize, Serialize};


#[derive(Serialize)]
pub struct CreateTransferRecipientRequest<'a> {
    #[serde(rename = "type")]
    pub recipient_type: &'a str,
    pub name: &'a str,
    pub account_number: &'a str,
    pub bank_code: &'a str,
    pub currency: CurrencyCode,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransferRecipientResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<RecipientData>,
}

#[derive(Debug, Deserialize)]
pub struct RecipientData {
    pub recipient_code: String,
}
