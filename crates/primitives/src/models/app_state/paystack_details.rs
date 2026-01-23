use eyre::eyre;
use eyre::Report;
use secrecy::SecretString;
use std::env;

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
                    .map_err(|_| eyre!("PAYSTACK_SECRET_KEY must be set"))?
                    .into(),
            ),
            paystack_api_url: env::var("PAYSTACK_API_URL")
                .unwrap_or_else(|_| "https://api.paystack.co".into()),

            paystack_webhook_secret: SecretString::new(
                env::var("PAYSTACK_WEBHOOK_SECRET")
                    .map_err(|_| eyre!("PAYSTACK_WEBHOOK_SECRET must be set"))?
                    .into(),
            ),
        })
    }
}
