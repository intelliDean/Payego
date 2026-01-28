use payego_primitives::models::app_state::stripe_details::StripeInfo;
use payego_primitives::error::ApiError;
use secrecy::ExposeSecret;
use stripe::{Client, CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionPaymentIntentData, PaymentIntent};
use std::collections::HashMap;

#[derive(Clone)]
pub struct StripeClient {
    client: Client,
}

impl StripeClient {
    pub fn new(config: &StripeInfo) -> Self {
        let client = Client::new(config.stripe_secret_key.expose_secret());
        Self { client }
    }

    pub async fn create_checkout_session(
        &self,
        amount: i64,
        currency: &str,
        transaction_ref: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, ApiError> {
        let mut metadata = HashMap::new();
        metadata.insert("transaction_reference".to_string(), transaction_ref.to_string());

        let line_item = CreateCheckoutSessionLineItems {
            quantity: Some(1),
            price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
                currency: currency.parse().map_err(|_| ApiError::Internal("Invalid currency for Stripe".into()))?,
                product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: "Wallet Top-up".to_string(),
                    ..Default::default()
                }),
                unit_amount: Some(amount),
                ..Default::default()
            }),
            ..Default::default()
        };

        let params = CreateCheckoutSession {
            mode: Some(CheckoutSessionMode::Payment),
            line_items: Some(vec![line_item]),
            success_url: Some(success_url),
            cancel_url: Some(cancel_url),
            payment_intent_data: Some(CreateCheckoutSessionPaymentIntentData {
                metadata: Some(metadata.clone()),
                ..Default::default()
            }),
            metadata: Some(metadata),
            ..Default::default()
        };

        CheckoutSession::create(&self.client, params)
            .await
            .map_err(|e| ApiError::Payment(format!("Stripe error: {}", e)))
    }

    pub async fn get_payment_intent(&self, id: &stripe::PaymentIntentId) -> Result<PaymentIntent, ApiError> {
        PaymentIntent::retrieve(&self.client, id, &[])
            .await
            .map_err(|e| ApiError::Payment(format!("Stripe error: {}", e)))
    }
}
