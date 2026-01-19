use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use crate::models::app_config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub http_client: Client,
    pub config: AppConfig,
}

use eyre::Result;

impl AppState {
    pub fn new(db: DbPool, config: AppConfig) -> Result<Arc<Self>> {
        let http = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Arc::new(Self { db, http_client: http, config }))
    }
}

// #[derive(Clone)]
// pub struct AppState1 {
//     pub db: DbPool,
//     pub jwt_secret: SecretString,
//     pub jwt_expiration_hours: i64,
//     pub jwt_issuer: String,
//     pub jwt_audience: String,
//     pub client: Client,
//     pub stripe_secret_key: SecretString,
//     pub app_url: String,
//     pub exchange_api_url: String,
//     pub paypal_api_url: String,
//     pub stripe_api_url: String,
//     pub stripe_webhook_secret: SecretString,
//     pub paystack_api_url: String,
//     pub paystack_secret_key: SecretString,
//     pub paystack_webhook_secret: SecretString,
//     pub paypal_client_id: String,
//     pub paypal_secret: SecretString,
// }

// impl AppState1 {
//     pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Result<Arc<AppState1>, Report> {
//         let state = Arc::new(AppState1 {
//             db: pool,
//             jwt_secret: SecretString::new(
//                 crate::config::security_config::JWT::new()
//                     .jwt_secret
//                     .into(),
//             ),
//             jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
//                 .unwrap_or_else(|_| "2".to_string())
//                 .parse()
//                 .map_err(|e| {
//                     error!("JWT expiration config error: {}", e);
//                     ApiError::Token(format!("Invalid JWT expiration configuration: {}", e))
//                 })?,
//             jwt_issuer: env::var("ISSUER")
//                 .unwrap_or_else(|_| "2".to_string())
//                 .parse()
//                 .map_err(|e| {
//                     error!("Issuer environment variable not set: {}", e);
//                     ApiError::Token(format!("Issuer environment variable not set: {}", e))
//                 })?,
//             jwt_audience: env::var("AUDIENCE")
//                 .unwrap_or_else(|_| "2".to_string())
//                 .parse()
//                 .map_err(|e| {
//                     error!("Audience environment variable not set: {}", e);
//                     ApiError::Token(format!("Audience environment variable not set: {}", e))
//                 })?,
//             client: Client::builder()
//                 .timeout(std::time::Duration::from_secs(10))
//                 .build()
//                 .map_err(|e| ApiError::Payment(e.to_string()))?,
//             stripe_secret_key: SecretString::new(
//                 env::var("STRIPE_SECRET_KEY")
//                     .map_err(|e| {
//                         error!("STRIPE_SECRET_KEY environment variable not set: {}", e);
//                         eyre::eyre!("STRIPE_SECRET_KEY environment variable must be set")
//                     })?
//                     .into(),
//             ),
//             paystack_secret_key: SecretString::new(
//                 env::var("PAYSTACK_SECRET_KEY")
//                     .map_err(|e| {
//                         error!("PAYSTACK_SECRET_KEY environment variable not set: {}", e);
//                         eyre::eyre!("PAYSTACK_SECRET_KEY environment variable must be set")
//                     })?
//                     .into(),
//             ),
//             app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
//             exchange_api_url: env::var("EXCHANGE_API_URL")
//                 .unwrap_or_else(|_| "https://api.exchangerate-api.com/v4/latest".to_string()),
//             paypal_api_url: env::var("PAYPAL_API_URL")
//                 .unwrap_or_else(|_| "https://api-m.sandbox.paypal.com".to_string()),
//             stripe_api_url: env::var("STRIPE_API_URL")
//                 .unwrap_or_else(|_| "https://api.stripe.com".to_string()),
//             stripe_webhook_secret: SecretString::new(
//                 env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default().into(),
//             ),
//             paystack_api_url: env::var("PAYSTACK_API_URL")
//                 .unwrap_or_else(|_| "https://api.paystack.co".to_string()),
//             paypal_client_id: env::var("PAYPAL_CLIENT_ID").unwrap_or_default(),
//             paypal_secret: SecretString::new(env::var("PAYPAL_SECRET").unwrap_or_default().into()),
//             paystack_webhook_secret: SecretString::new(
//                 env::var("PAYSTACK_WEBHOOK_SECRET")
//                     .unwrap_or_default()
//                     .into(),
//             ),
//         });
//         Ok(state)
//     }
// }
