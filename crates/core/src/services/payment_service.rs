pub use crate::app_state::AppState;
use crate::repositories::transaction_repository::TransactionRepository;
pub use crate::security::Claims;
use crate::services::audit_service::AuditService;
// use diesel::prelude::*;
pub use payego_primitives::{
    error::ApiError,
    models::{
        dtos::wallet_dto::{TopUpRequest, TopUpResponse},
        enum_types::{PaymentProvider, PaymentState, TransactionIntent},
        transaction::NewTransaction,
    },
};
use secrecy::ExposeSecret;
use tracing::{error, info};
use uuid::Uuid;

pub struct PaymentService;

impl PaymentService {
    pub async fn initiate_top_up(
        state: &AppState,
        user_id: Uuid,
        req: TopUpRequest,
    ) -> Result<TopUpResponse, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let tx_ref = Uuid::new_v4();

        // Create initial transaction
        let transaction = TransactionRepository::create(
            &mut conn,
            NewTransaction {
                user_id,
                counterparty_id: None,
                intent: TransactionIntent::TopUp,
                amount: req.amount,
                currency: req.currency,
                txn_state: PaymentState::Pending,
                provider: Some(req.provider),
                provider_reference: None,
                idempotency_key: &req.idempotency_key,
                reference: tx_ref,
                description: Some("Wallet top-up initiation"),
                metadata: serde_json::json!({
                    "idempotency_key": req.idempotency_key,
                }),
            },
        )?;

        let tx_ref = transaction.reference;

        let response = match req.provider {
            PaymentProvider::Stripe => {
                let success_url =
                    format!("{}/success?transaction_id={}", state.config.app_url, tx_ref);
                let cancel_url = format!("{}/top-up", state.config.app_url);

                let session = state
                    .stripe
                    .create_checkout_session(
                        req.amount,
                        &req.currency.to_string().to_lowercase(),
                        &tx_ref.to_string(),
                        &success_url,
                        &cancel_url,
                    )
                    .await?;

                info!("Stripe session created: {}", session.id);
                TopUpResponse {
                    session_url: session.url,
                    payment_id: Some(session.id.to_string()),
                    transaction_id: tx_ref.to_string(),
                    amount: req.amount,
                }
            }
            PaymentProvider::Paypal => {
                let amount_str = format!("{}.{:02}", req.amount / 100, req.amount % 100);

                let paypal_res = state
                    .http_client
                    .post(format!("{}/v2/checkout/orders", state.config.paypal_details.paypal_api_url))
                    .basic_auth(
                        state.config.paypal_details.paypal_client_id.clone(),
                        Some(state.config.paypal_details.paypal_secret.expose_secret()),
                    )
                    .json(&serde_json::json!({
                        "intent": "CAPTURE",
                        "purchase_units": [{
                            "reference_id": tx_ref.to_string(),
                            "amount": {
                                "currency_code": req.currency.to_string(),
                                "value": amount_str
                            }
                        }],
                        "application_context": {
                            "return_url": format!("{}/success?transaction_id={}", state.config.app_url, tx_ref),
                            "cancel_url": format!("{}/top-up", state.config.app_url)
                        }
                    }))
                    .send()
                    .await
                    .map_err(|e| ApiError::Payment(format!("PayPal request failed: {}", e)))?;

                let body: serde_json::Value = paypal_res
                    .json()
                    .await
                    .map_err(|_| ApiError::Payment("Invalid PayPal response".into()))?;

                let approval_url = body["links"]
                    .as_array()
                    .and_then(|links| {
                        links
                            .iter()
                            .find(|l| l["rel"] == "approve")
                            .and_then(|l| l["href"].as_str())
                    })
                    .ok_or_else(|| {
                        error!("PayPal Approval link missing. Body: {:?}", body);
                        ApiError::Payment("PayPal approval link missing".into())
                    })?;

                TopUpResponse {
                    session_url: Some(approval_url.to_string()),
                    payment_id: body["id"].as_str().map(|s| s.to_string()),
                    transaction_id: tx_ref.to_string(),
                    amount: req.amount,
                }
            }
            _ => return Err(ApiError::BadRequest("Unsupported provider".into())),
        };

        let _ = AuditService::log_event(
            state,
            Some(user_id),
            "payment.top_up.initiated",
            Some("transaction"),
            Some(&tx_ref.to_string()),
            serde_json::json!({
                "amount": req.amount,
                "currency": req.currency,
                "provider": req.provider,
            }),
            None,
        )
        .await;

        Ok(response)
    }
}
