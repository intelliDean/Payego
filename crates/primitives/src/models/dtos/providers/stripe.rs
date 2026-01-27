use uuid::Uuid;

#[derive(Debug)]
pub struct StripeWebhookContext {
    pub transaction_ref: Uuid,
    pub provider_reference: String,
    pub currency: String,
}
