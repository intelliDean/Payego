use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaystackWebhook {
    pub event: String,
    pub data: PaystackData,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaystackData {
    pub reference: String,
    pub currency: Option<String>,
}

#[derive(Deserialize)]
pub struct PaystackRecipientResponse {
    pub status: bool,
    pub data: PaystackRecipientData,
}

#[derive(Deserialize)]
pub struct PaystackRecipientData {
    pub recipient_code: String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackAccountData {
    pub account_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct PaystackBank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "active", default)]
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct PaystackResolveResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<PaystackAccountData>,
}

#[derive(Deserialize)]
pub struct PaystackTransferData {
    pub transfer_code: String,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct PaystackTransferResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<PaystackTransferData>,
}

#[derive(Debug, Deserialize)]
pub struct PaystackTransData {
    pub transfer_code: String,
    pub reference: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaystackResponseWrapper<T> {
    pub status: bool,
    pub message: String,
    pub data: Option<T>,
}
