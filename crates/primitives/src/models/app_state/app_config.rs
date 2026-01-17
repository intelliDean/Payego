
use crate::models::app_state::paypal_details::PaypalInfo;
use crate::models::app_state::paystack_details::PaystackInfo;
use crate::models::app_state::stripe_details::StripeInfo;
use eyre::Report;
use std::env;
use crate::models::app_state::jwt_details::JWTInfo;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jwt_details: JWTInfo,

    pub app_url: String,

    pub conversion_fee_bps: i128,

    pub stripe_details: StripeInfo,

    pub paystack_details: PaystackInfo,

    pub paypal_details: PaypalInfo,

    pub exchange_api_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Report> {
        Ok(Self {
            jwt_details: JWTInfo::new()?,

            app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".into()),

            conversion_fee_bps: env::var("FEE_BPS").unwrap_or_else(|_| "100".into()).parse()?,
            stripe_details: StripeInfo::new()?,

            paystack_details: PaystackInfo::new()?,

            paypal_details: PaypalInfo::new()?,

            exchange_api_url: env::var("EXCHANGE_API_URL")
                .unwrap_or_else(|_| "https://api.exchangerate-api.com/v4/latest".into()),
        })
    }
}
