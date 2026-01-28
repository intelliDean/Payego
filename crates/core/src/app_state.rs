use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use crate::clients::{EmailClient, ExchangeRateClient, PaystackClient, StripeClient};
use eyre::Result;
pub use payego_primitives::models::app_config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub http_client: Client,
    pub config: AppConfig,
    pub paystack: PaystackClient,
    pub stripe: StripeClient,
    pub fx: ExchangeRateClient,
    pub email: EmailClient,
}

impl AppState {
    pub fn new(db: DbPool, config: AppConfig) -> Result<Arc<Self>> {
        let http = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let paystack = PaystackClient::new(
            http.clone(),
            &config.paystack_details.paystack_api_url,
            config.paystack_details.paystack_secret_key.clone(),
        )?;

        let stripe = StripeClient::new(&config.stripe_details);

        let fx = ExchangeRateClient::new(http.clone(), &config.exchange_api_url)?;

        let email = EmailClient::new();

        Ok(Arc::new(Self {
            db,
            http_client: http,
            config,
            paystack,
            stripe,
            fx,
            email,
        }))
    }
}
