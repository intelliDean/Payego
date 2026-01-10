use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, NewTransaction, Transaction, TopUpRequest, TopUpResponse};
use reqwest::Client;
use secrecy::ExposeSecret;
use std::str::FromStr;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client as StripeClient, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData,
    CreateCheckoutSessionLineItemsPriceDataProductData, Currency,
};
use tracing::{error, info};
use uuid::Uuid;

pub struct PaymentService;

impl PaymentService {
    pub async fn initiate_top_up(
        state: &AppState,
        user_id: Uuid,
        req: TopUpRequest,
    ) -> Result<TopUpResponse, ApiError> {
        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // 1. Idempotency check with metadata
        let existing_transaction = payego_primitives::schema::transactions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ")
                    .bind::<diesel::sql_types::Text, _>(&req.idempotency_key),
            )
            .filter(payego_primitives::schema::transactions::user_id.eq(user_id))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(|e: diesel::result::Error| {
                error!("Database error checking idempotency: {}", e);
                ApiError::from(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!(
                "Idempotent request: transaction {} already exists for key {}",
                tx.reference, req.idempotency_key
            );
            return Ok(TopUpResponse {
                session_url: None,
                payment_id: tx
                    .provider
                    .as_ref()
                    .filter(|p| p.as_str() == "paypal")
                    .map(|_| tx.id.to_string()),
                transaction_id: tx.reference.to_string(),
                amount: (tx.amount as f64) / 100.0,
            });
        }

        let amount_cents = (req.amount * 100.0).round() as i64;
        let transaction_id = req.reference;

        // 2. Insert pending transaction
        conn.transaction::<(), ApiError, _>(|conn| {
            diesel::insert_into(payego_primitives::schema::transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: amount_cents,
                    transaction_type: format!("topup_{}", req.provider),
                    currency: req.currency.to_uppercase(),
                    status: "pending".to_string(),
                    provider: Some(req.provider.clone()),
                    description: Some(format!("{} top-up in {}", req.provider, req.currency)),
                    reference: transaction_id,
                    metadata: Some(serde_json::json!({
                        "idempotency_key": req.idempotency_key
                    })),
                })
                .execute(conn)
                .map_err(ApiError::from)?;

            Ok(())
        })?;

        let client = Client::new();
        let (session_url, payment_id) = match req.provider.as_str() {
            "stripe" => {
                Self::initiate_stripe_payment(&state, &req, transaction_id, amount_cents).await?
            }
            "paypal" => {
                Self::initiate_paypal_payment(&client, &state, &req, transaction_id).await?
            }
            _ => {
                return Err(ApiError::Payment(format!(
                    "Unsupported provider: {}",
                    req.provider
                )))
            }
        };

        info!(
            "Top-up initiated for user {}: transaction {}, amount {}, currency {}, provider {}",
            user_id, transaction_id, req.amount, req.currency, req.provider
        );

        Ok(TopUpResponse {
            session_url,
            payment_id,
            transaction_id: transaction_id.to_string(),
            amount: req.amount,
        })
    }

    async fn initiate_stripe_payment(
        state: &AppState,
        req: &TopUpRequest,
        transaction_id: Uuid,
        amount_cents: i64,
    ) -> Result<(Option<String>, Option<String>), ApiError> {
        let mut stripe_client = StripeClient::new(state.stripe_secret_key.expose_secret());
        if !state.stripe_api_url.is_empty() && state.stripe_api_url != "https://api.stripe.com" {
            stripe_client = StripeClient::from_url(state.stripe_api_url.as_str(), state.stripe_secret_key.expose_secret());
        }
        let currency_val = Currency::from_str(&req.currency.to_lowercase())
            .map_err(|_| ApiError::Payment(format!("Invalid currency: {}", req.currency)))?;

        let line_item = CreateCheckoutSessionLineItems {
            quantity: Some(1),
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency: currency_val,
                unit_amount: Some(amount_cents),
                product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: "Account Top-Up".to_string(),
                    description: Some(format!(
                        "Add {} {} to your account",
                        req.amount, req.currency
                    )),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("transaction_id".to_string(), transaction_id.to_string());

        let session_params = CreateCheckoutSession {
            success_url: Some(&format!(
                "{}/success?transaction_id={}",
                state.app_url, transaction_id
            )),
            cancel_url: Some(&format!("{}/top-up", state.app_url)),
            mode: Some(CheckoutSessionMode::Payment),
            line_items: Some(vec![line_item]),
            metadata: Some(metadata),
            ..Default::default()
        };

        let session = CheckoutSession::create(&stripe_client, session_params)
            .await
            .map_err(|e: stripe::StripeError| {
                error!("Stripe error: {}", e);
                ApiError::Payment(format!("Stripe error: {}", e))
            })?;

        Ok((session.url, None))
    }

    async fn initiate_paypal_payment(
        client: &Client,
        state: &AppState,
        req: &TopUpRequest,
        transaction_id: Uuid,
    ) -> Result<(Option<String>, Option<String>), ApiError> {
        let client_id = &state.paypal_client_id;
        let secret = state.paypal_secret.expose_secret();
        let app_url = &state.app_url;
        let base_url = &state.paypal_api_url;

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
                        "currency_code": req.currency.to_uppercase(),
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
