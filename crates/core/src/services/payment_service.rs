use diesel::prelude::*;

use payego_primitives::models::enum_types::CurrencyCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;
use validator::Validate;

use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::{
    app_state::app_state::AppState,
    enum_types::{PaymentProvider, PaymentState, TransactionIntent},
    transaction::{NewTransaction, Transaction},
};
use payego_primitives::schema::transactions;
use reqwest::Client;
use secrecy::ExposeSecret;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client as StripeClient, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData,
    CreateCheckoutSessionLineItemsPriceDataProductData, Currency,
};
use tracing::error;
use uuid::Uuid;
use payego_primitives::models::top_up_dto::{TopUpRequest, TopUpResponse};

pub struct PaymentService;

impl PaymentService {
    pub async fn initiate_top_up(
        state: &AppState,
        user_id: Uuid,
        req: TopUpRequest,
    ) -> Result<TopUpResponse, ApiError> {
        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let amount_cents = Self::convert_to_cents(req.amount)?;
        let reference = Uuid::new_v4();

        // ---------- DB-ENFORCED IDEMPOTENCY ----------
        let inserted = diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id,
                counterparty_id: None,
                intent: TransactionIntent::TopUp,
                amount: amount_cents,
                currency: req.currency,
                txn_state: PaymentState::Pending,
                provider: Some(req.provider),
                provider_reference: None,
                idempotency_key: &req.idempotency_key,
                reference,
                description: Some("Top-up intent"),
                metadata: serde_json::json!({}),
            })
            .on_conflict((transactions::user_id, transactions::idempotency_key))
            .do_nothing()
            .returning(transactions::id)
            .get_result::<Uuid>(&mut conn)
            .optional()?;

        let tx = match inserted {
            Some(_) => transactions::table
                .filter(transactions::reference.eq(reference))
                .first::<Transaction>(&mut conn)?,

            None => transactions::table
                .filter(transactions::user_id.eq(user_id))
                .filter(transactions::idempotency_key.eq(&req.idempotency_key))
                .first::<Transaction>(&mut conn)?,
        };

        let (session_url, payment_id) = match req.provider {
            PaymentProvider::Stripe => {
                Self::initiate_stripe(state, &req, tx.reference, amount_cents).await?
            }
            PaymentProvider::Paypal => {
                let client = Client::new();
                Self::initiate_paypal(&client, state, &req, tx.reference).await?
            }
            _ => return Err(ApiError::Payment("Unsupported provider".into())),
        };

        Ok(TopUpResponse {
            session_url,
            payment_id,
            transaction_id: tx.reference.to_string(),
            amount: req.amount,
        })
    }

    fn convert_to_cents(amount: f64) -> Result<i64, ApiError> {
        if amount <= 0.0 {
            return Err(ApiError::Payment("Amount must be positive".into()));
        }
        Ok((amount * 100.0).round() as i64)
    }

    async fn initiate_stripe(
        state: &AppState,
        req: &TopUpRequest,
        reference: Uuid,
        amount_cents: i64,
    ) -> Result<(Option<String>, Option<String>), ApiError> {
        let client = StripeClient::new(
            state
                .config
                .stripe_details
                .stripe_secret_key
                .expose_secret(),
        );

        let currency = Currency::from_str(&req.currency.to_string().to_lowercase())
            .map_err(|_| ApiError::Payment("Invalid currency".into()))?;

        let session = CheckoutSession::create(
            &client,
            CreateCheckoutSession {
                mode: Some(CheckoutSessionMode::Payment),
                success_url: Some(&format!(
                    "{}/success?tx={}",
                    state.config.app_url, reference
                )),
                cancel_url: Some(&format!("{}/top-up", state.config.app_url)),
                line_items: Some(vec![CreateCheckoutSessionLineItems {
                    quantity: Some(1),
                    price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                        currency,
                        unit_amount: Some(amount_cents),
                        product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                            name: "Account Top-Up".into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                metadata: Some(
                    [("transaction_ref".into(), reference.to_string())]
                        .into_iter()
                        .collect(),
                ),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| ApiError::Payment(e.to_string()))?;

        Ok((session.url, None))
    }

    async fn initiate_paypal(
        client: &Client,
        state: &AppState,
        req: &TopUpRequest,
        transaction_id: Uuid,
    ) -> Result<(Option<String>, Option<String>), ApiError> {
        let client_id = &state.config.paypal_details.paypal_client_id;
        let secret = state.config.paypal_details.paypal_secret.expose_secret();
        let app_url = &state.config.app_url;
        let base_url = &state.config.paypal_details.paypal_api_url;

        let token_resp = client
            .post(format!("{}/v1/oauth2/token", base_url))
            .header("Content_Type", "application/x-www-form-urlencoded")
            .basic_auth(client_id, Some(secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("PayPal auth error: {}", e)))?;

        let token_json = token_resp
            .json::<serde_json::Value>()
            .await
            .map_err(|_| ApiError::Payment("PayPal token parse error".into()))?;
        let access_token = token_json["access_token"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("PayPal token missing".into()))?;

        let resp = client
            .post(format!("{}/v2/checkout/orders", base_url))
            .bearer_auth(access_token)
            .json(&serde_json::json!({
                "intent": "CAPTURE",
                "purchase_units": [{
                    "amount": {
                        "currency_code": req.currency,
                        "value": format!("{:.2}", req.amount),
                    },
                    "description": format!("Top-up {}", transaction_id),
                    "custom_id": transaction_id.to_string(),
                }],
                "application_context": {
                    "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
                    "return_url": format!("{}/success?transaction_id={}", app_url, transaction_id),
                    "cancel_url": format!("{}/top-up", app_url),
                    "brand_name": "Payego",
                    "user_action": "PAY_NOW"
                }
            }))
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("PayPal order error: {}", e)))?;

        let json = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|_| ApiError::Payment("PayPal response parse error".into()))?;
        let payment_id = json["id"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("PayPal ID missing".into()))?;

        let session_url = json["links"]
            .as_array()
            .and_then(|links| {
                links
                    .iter()
                    .find(|l| l["rel"] == "approve")
                    .and_then(|l| l["href"].as_str())
            })
            .map(|s| s.to_string());

        Ok((session_url, Some(payment_id.to_string())))
    }
}
