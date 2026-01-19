use crate::models::enum_types::{CurrencyCode, PaymentState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;


#[derive(Deserialize, ToSchema)]
pub struct CaptureRequest {
    pub order_id: String,
    pub transaction_id: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct CaptureResponse {
    pub status: PaymentState,
    pub transaction_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

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

#[derive(Debug, Deserialize)]
pub struct PayPalTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct PayPalOrderResponse {
    pub status: String,
}

pub struct PaypalCapture {
    pub capture_id: String,
    pub currency: CurrencyCode,
}

#[derive(Debug)]
pub struct StripeWebhookContext {
    pub transaction_ref: Uuid,
    pub provider_reference: String,
    pub currency: String,
}


#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub status: String,
}
