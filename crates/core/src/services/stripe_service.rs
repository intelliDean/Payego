use axum::body::Bytes;
use http::HeaderMap;
use secrecy::ExposeSecret;
use std::sync::Arc;
use stripe::{Event, EventObject, EventType, Webhook};
use uuid::Uuid;

pub use payego_primitives::{
    error::ApiError,
    models::{
        app_state::app_state::AppState, providers_dto::StripeWebhookContext
    }
};

pub struct StripeService;

impl StripeService {
    pub fn construct_event(
        state: &Arc<AppState>,
        headers: HeaderMap,
        body: &Bytes,
    ) -> Result<Event, ApiError> {
        let signature = headers
            .get("stripe-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Payment("Missing Stripe signature".into()))?;

        let payload_str = std::str::from_utf8(&body)
            .map_err(|_| ApiError::Payment("Invalid UTF-8 Stripe payload".into()))?;

        Webhook::construct_event(
            payload_str,
            signature,
            state
                .config
                .stripe_details
                .stripe_webhook_secret
                .expose_secret(),
        )
        .map_err(|e| ApiError::Payment(format!("Invalid Stripe webhook: {}", e)))
    }

    pub fn extract_context(
        state: &Arc<AppState>,
        headers: HeaderMap,
        body: &Bytes,
    ) -> Result<Option<StripeWebhookContext>, ApiError> {
        let event = Self::construct_event(&state, headers, &body)?;

        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                let EventObject::CheckoutSession(session) = &event.data.object else {
                    return Err(ApiError::Payment("Unexpected Stripe object".into()));
                };

                let metadata = session
                    .metadata
                    .as_ref()
                    .ok_or(ApiError::Payment("Missing Stripe metadata".into()))?;

                let tx_ref = metadata
                    .get("transaction_reference")
                    .ok_or(ApiError::Payment("Missing transaction_reference".into()))?;

                let transaction_ref = Uuid::parse_str(tx_ref)
                    .map_err(|_| ApiError::Payment("Invalid transaction_reference".into()))?;

                let currency = session
                    .currency
                    .map(|c| c.to_string())
                    .ok_or_else(|| ApiError::Payment("Stripe session missing currency".into()))?;

                let provider_reference = session.id.to_string(); // Stripe IDs are strings

                Ok(Some(StripeWebhookContext {
                    transaction_ref,
                    provider_reference,
                    currency,
                }))
            }
            _ => Ok(None),
        }
    }
}
