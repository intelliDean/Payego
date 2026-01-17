// payego_core::services::stripe_service.rs

use stripe::{Event, EventObject, EventType, Webhook};
use uuid::Uuid;

use payego_primitives::error::ApiError;

#[derive(Debug)]
pub struct StripeWebhookContext {
    pub transaction_ref: Uuid,
    pub provider_reference: String,
    pub currency: String,
}

pub struct StripeService;

impl StripeService {
    pub fn construct_event(
        payload: &str,
        signature: &str,
        secret: &str,
    ) -> Result<Event, ApiError> {
        Webhook::construct_event(payload, signature, secret)
            .map_err(|e| ApiError::Payment(format!("Invalid Stripe webhook: {}", e)))
    }

    pub fn extract_context(event: &Event) -> Result<Option<StripeWebhookContext>, ApiError> {
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
                    .ok_or_else(|| {
                        ApiError::Payment("Stripe session missing currency".into())
                    })?;

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
