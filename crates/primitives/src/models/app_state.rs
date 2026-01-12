use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::PgConnection;
use eyre::Report;
use std::env;
use std::sync::Arc;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use crate::error::ApiError;
use secrecy::SecretString;
use tracing::error;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: SecretString,
    pub jwt_expiration_hours: i64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub stripe_secret_key: SecretString,
    pub app_url: String,
    pub exchange_api_url: String,
    pub paypal_api_url: String,
    pub stripe_api_url: String,
    pub paystack_api_url: String,
    pub paystack_secret_key: SecretString,
    pub paypal_client_id: String,
    pub paypal_secret: SecretString,
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Result<Arc<AppState>, Report> {
        let state = Arc::new(AppState {
            db: pool,
            jwt_secret: SecretString::new(
                crate::config::security_config::JWTSecret::new()
                    .jwt_secret
                    .into(),
            ),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .map_err(|e| {
                    error!("JWT expiration config error: {}", e);
                    ApiError::Token(format!("Invalid JWT expiration configuration: {}", e))
                })?,
            jwt_issuer: env::var("ISSUER")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .map_err(|e| {
                    error!("Issuer environment variable not set: {}", e);
                    ApiError::Token(format!("Issuer environment variable not set: {}", e))
                })?,
            jwt_audience: env::var("AUDIENCE")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .map_err(|e| {
                    error!("Audience environment variable not set: {}", e);
                    ApiError::Token(format!("Audience environment variable not set: {}", e))
                })?,
            stripe_secret_key: SecretString::new(
                env::var("STRIPE_SECRET_KEY")
                    .map_err(|e| {
                        error!("STRIPE_SECRET_KEY environment variable not set: {}", e);
                        eyre::eyre!("STRIPE_SECRET_KEY environment variable must be set")
                    })?
                    .into(),
            ),
            paystack_secret_key: SecretString::new(
                env::var("PAYSTACK_SECRET_KEY")
                    .map_err(|e| {
                        error!("PAYSTACK_SECRET_KEY environment variable not set: {}", e);
                        eyre::eyre!("PAYSTACK_SECRET_KEY environment variable must be set")
                    })?
                    .into(),
            ),
            app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            exchange_api_url: env::var("EXCHANGE_API_URL")
                .unwrap_or_else(|_| "https://api.exchangerate-api.com/v4/latest".to_string()),
            paypal_api_url: env::var("PAYPAL_API_URL")
                .unwrap_or_else(|_| "https://api-m.sandbox.paypal.com".to_string()),
            stripe_api_url: env::var("STRIPE_API_URL")
                .unwrap_or_else(|_| "https://api.stripe.com".to_string()),
            paystack_api_url: env::var("PAYSTACK_API_URL")
                .unwrap_or_else(|_| "https://api.paystack.co".to_string()),
            paypal_client_id: env::var("PAYPAL_CLIENT_ID").unwrap_or_default(),
            paypal_secret: SecretString::new(env::var("PAYPAL_SECRET").unwrap_or_default().into()),
        });
        Ok(state)
    }
}
