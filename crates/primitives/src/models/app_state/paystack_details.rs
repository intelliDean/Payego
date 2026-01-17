
use crate::error::ApiError;
use crate::models::app_state::stripe_details::StripeInfo;
use secrecy::SecretString;
use std::env;
use eyre::Report;

#[derive(Debug, Clone)]
pub struct PaystackInfo {
    pub paystack_secret_key: SecretString,
    pub paystack_api_url: String,
    pub paystack_webhook_secret: SecretString,
}

impl PaystackInfo {
    pub fn new() -> Result<Self, Report> {
        Ok(Self {
            paystack_secret_key: SecretString::new(
                env::var("PAYSTACK_SECRET_KEY")
                    .map_err(|_| ApiError::Token("PAYSTACK_SECRET_KEY must be set".into()))?
                    .into(),
            ),
            paystack_api_url: env::var("PAYSTACK_API_URL")
                .unwrap_or_else(|_| "https://api.paystack.co".into()),

            paystack_webhook_secret: SecretString::new(
                env::var("PAYSTACK_WEBHOOK_SECRET")
                    .map_err(|_| ApiError::Token("PAYSTACK_WEBHOOK_SECRET must be set".into()))?
                    .into(),
            ),
        })
    }
}
