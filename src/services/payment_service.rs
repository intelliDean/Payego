use diesel::prelude::*;
use uuid::Uuid;
use std::sync::Arc;
use crate::error::ApiError;
use crate::models::models::{AppState, NewTransaction, Transaction};
use crate::handlers::top_up::{TopUpRequest, TopUpResponse};
use tracing::{error, info};
use reqwest::Client;
use stripe::{CheckoutSession, CheckoutSessionMode, Client as StripeClient, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData, Currency};
use std::str::FromStr;
use serde_json;
use secrecy::ExposeSecret;

pub struct PaymentService;

impl PaymentService {
    pub async fn initiate_top_up(
        state: Arc<AppState>,
        user_id: Uuid,
        req: TopUpRequest,
    ) -> Result<TopUpResponse, ApiError> {
        let conn = &mut state.db.get().map_err(|e| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // 1. Idempotency check
        // 1. Idempotency check with metadata
        let existing_transaction = crate::schema::transactions::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ").bind::<diesel::sql_types::Text, _>(&req.idempotency_key))
            .filter(crate::schema::transactions::user_id.eq(user_id))
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| {
                error!("Database error checking idempotency: {}", e);
                ApiError::Database(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!("Idempotent request: transaction {} already exists for key {}", tx.reference, req.idempotency_key);
            return Ok(TopUpResponse {
                session_url: None,
                payment_id: tx.provider.as_ref().filter(|p| p.as_str() == "paypal").map(|_| tx.id.to_string()),
                transaction_id: tx.reference.to_string(),
                amount: (tx.amount as f64) / 100.0,
            });
        }

        let amount_cents = (req.amount * 100.0).round() as i64;
        let transaction_id = req.reference;

        // 2. Insert pending transaction
        conn.transaction(|conn| {
            diesel::insert_into(crate::schema::transactions::table)
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
                .map_err(ApiError::Database)?;

            Ok::<(), ApiError>(())
        })?;

        let client = Client::new();
        let (session_url, payment_id) = match req.provider.as_str() {
            "stripe" => {
                Self::initiate_stripe_payment(&state, &req, transaction_id, amount_cents).await?
            }
            "paypal" => {
                Self::initiate_paypal_payment(&client, &state, &req, transaction_id).await?
            }
            _ => return Err(ApiError::Payment(format!("Unsupported provider: {}", req.provider))),
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
        let stripe_client = StripeClient::new(state.stripe_secret_key.expose_secret());
        let line_item = CreateCheckoutSessionLineItems {
            quantity: Some(1),
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency: Currency::from_str(&req.currency.to_lowercase()).map_err(|e| {
                    ApiError::Payment(format!("Invalid currency: {}", e))
                })?,
                unit_amount: Some(amount_cents),
                product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: "Account Top-Up".to_string(),
                    description: Some(format!("Add {} {} to your account", req.amount, req.currency)),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("transaction_id".to_string(), transaction_id.to_string());

        let session_params = CreateCheckoutSession {
            success_url: Some(&format!("{}/success?transaction_id={}", state.app_url, transaction_id)),
            cancel_url: Some(&format!("{}/top-up", state.app_url)),
            mode: Some(CheckoutSessionMode::Payment),
            line_items: Some(vec![line_item]),
            metadata: Some(metadata),
            ..Default::default()
        };

        let session = CheckoutSession::create(&stripe_client, session_params)
            .await
            .map_err(|e| {
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
        let client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|_| ApiError::Payment("PayPal ID missing".into()))?;
        let secret = std::env::var("PAYPAL_SECRET").map_err(|_| ApiError::Payment("PayPal secret missing".into()))?;
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let base_url = &state.paypal_api_url;

        let token_resp = client
            .post(format!("{}/v1/oauth2/token", base_url))
            .header("Content_Type", "application/x-www-form-urlencoded")
            .basic_auth(client_id, Some(secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("PayPal auth error: {}", e)))?;

        let token_json = token_resp.json::<serde_json::Value>().await.map_err(|_| ApiError::Payment("PayPal token parse error".into()))?;
        let access_token = token_json["access_token"].as_str().ok_or_else(|| ApiError::Payment("PayPal token missing".into()))?;

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
            .map_err(|e| ApiError::Payment(format!("PayPal order error: {}", e)))?;

        let json = resp.json::<serde_json::Value>().await.map_err(|_| ApiError::Payment("PayPal response parse error".into()))?;
        let payment_id = json["id"].as_str().ok_or_else(|| ApiError::Payment("PayPal ID missing".into()))?;

        Ok((None, Some(payment_id.to_string())))
    }
}
