use eyre::{eyre, Report};
use secrecy::SecretString;
use std::env;

#[derive(Debug, Clone)]
pub struct PaypalInfo {
    pub paypal_client_id: String,
    pub paypal_secret: SecretString,
    pub paypal_api_url: String,
}

impl PaypalInfo {
    pub fn new() -> Result<Self, Report> {
        Ok(Self {
            paypal_client_id: env::var("PAYPAL_CLIENT_ID")
                .map_err(|_| eyre!("PAYPAL_CLIENT_ID must be set"))?,

            paypal_secret: SecretString::new(
                env::var("PAYPAL_SECRET")
                    .map_err(|_| eyre!("PAYPAL_SECRET must be set"))?
                    .into(),
            ),
            paypal_api_url: env::var("PAYPAL_API_URL")
                .unwrap_or_else(|_| "https://api-m.sandbox.paypal.com".into()),
        })
    }
}
