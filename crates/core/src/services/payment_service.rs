use diesel::prelude::*;
use http::header::{CONTENT_TYPE, USER_AGENT};
use payego_primitives::models::providers_dto::{PayPalOrderResp, PayPalOrderResponse};
pub use payego_primitives::{
    config::security_config::Claims,
    error::ApiError,
    models::{
        app_state::app_state::AppState,
        enum_types::{PaymentProvider, PaymentState, TransactionIntent},
        top_up_dto::{TopUpRequest, TopUpResponse},
        transaction::{NewTransaction, Transaction},
    },
    schema::transactions,
};
use reqwest::{Client, Url};
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use std::time::Duration;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client as StripeClient, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData,
    CreateCheckoutSessionLineItemsPriceDataProductData, Currency,
};
use tracing::error;
use uuid::Uuid;

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
            PaymentProvider::Paypal => Self::initiate_paypal(state, &req, tx.reference).await?,
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
        let client = StripeClient::from_url(
            state.config.stripe_details.stripe_api_url.as_str(),
            state.config.stripe_details.stripe_secret_key.expose_secret(),
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
        state: &AppState,
        req: &TopUpRequest,
        transaction_id: Uuid,
    ) -> Result<(Option<String>, Option<String>), ApiError> {
        let client_id = &state.config.paypal_details.paypal_client_id;
        let secret = state.config.paypal_details.paypal_secret.expose_secret();
        let app_url = &state.config.app_url;
        let base_url = &state.config.paypal_details.paypal_api_url;

        //even though I know this will not happen because emptiness is dealt with from onset
        if client_id.trim().is_empty() || secret.trim().is_empty() {
            return Err(ApiError::Internal("PayPal credentials missing".into()));
        }

        let mut url = Url::parse(base_url)
            .map_err(|_| ApiError::Internal("Invalid PayPal base URL".into()))?;

        url.set_path("/v1/oauth2/token");

        let token_resp = state
            .http_client
            .post(url)
            .timeout(std::time::Duration::from_secs(5)) // i want this to override the default which is 10secs
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, "Payego/1.0 (Rust backend)")
            .basic_auth(client_id, Some(secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| {
                tracing::error!("PayPal OAuth request failed: {}", e);
                ApiError::Payment("PayPal authentication failed".into())
            })?;

        let token_json = token_resp
            .json::<serde_json::Value>()
            .await
            .map_err(|_| ApiError::Payment("PayPal token parse error".into()))?;
        let access_token = token_json["access_token"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("PayPal token missing".into()))?;

        let mut url = Url::parse(base_url)
            .map_err(|_| ApiError::Internal("Invalid PayPal base URL".into()))?;

        url.set_path("v2/checkout/orders");

        // Validate currency early (defensive)
        let currency = req.currency.to_string();
        if currency.len() != 3 {
            return Err(ApiError::Internal("Invalid currency code".into()));
        }

        // Pre-format amount safely
        let amount_str = format!("{:.2}", req.amount);
        if req.amount <= 0.0 {
            return Err(ApiError::Internal(
                "Amount must be greater than zero".into(),
            ));
        }

        let payload = json!({
            "intent": "CAPTURE",
            "purchase_units": [{
                "amount": {
                    "currency_code": currency,
                    "value": amount_str,
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
        });

        let resp = state
            .http_client
            .post(url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("PayPal order request failed: {}", e);
                ApiError::Payment("Failed to reach PayPal".into())
            })?;

        let body: PayPalOrderResp = resp
            .json()
            .await
            .map_err(|_| ApiError::Payment("Invalid PayPal response".into()))?;

        let approve_url = body
            .links
            .iter()
            .find(|l| l.rel == "approve")
            .ok_or_else(|| ApiError::Payment("Approval link missing".into()))?
            .href
            .clone();

        if body.id.trim().is_empty() {
            return Err(ApiError::Payment("Empty PayPal payment ID".into()));
        }

        Ok((Some(approve_url), Some(body.id.clone())))
    }
}
