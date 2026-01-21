use crate::error::ApiError;
use eyre::Report;
use secrecy::SecretString;
use std::env;

#[derive(Debug, Clone)]
pub struct StripeInfo {
    pub stripe_secret_key: SecretString,
    pub stripe_api_url: String,
    pub stripe_webhook_secret: SecretString,
}

impl StripeInfo {
    pub fn new() -> Result<Self, Report> {
        Ok(Self {
            stripe_secret_key: SecretString::new(
                env::var("STRIPE_SECRET_KEY")
                    .map_err(|_| {
                        ApiError::Token("STRIPE_SECRET_KEY environment variable must be set".into())
                    })?
                    .into(),
            ),
            stripe_api_url: env::var("STRIPE_API_URL")
                .unwrap_or_else(|_| "https://api.stripe.com".into()),

            stripe_webhook_secret: SecretString::new(
                env::var("STRIPE_WEBHOOK_SECRET")
                    .map_err(|_| ApiError::Token("STRIPE_WEBHOOK_SECRET must be set".into()))?
                    .into(),
            ),
        })
    }
}
