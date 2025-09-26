use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::models::models::{AppState, NewTransaction, Wallet};
use axum::{Extension, Json, extract::State, http::StatusCode};
use base64::{
    Engine as _, alphabet,
    engine::{self, general_purpose},
};
use diesel::prelude::*;
use jsonwebtoken::errors::ErrorKind::Base64;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, LazyLock};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

static SUPPORTED_PROVIDERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(stripe|paypal)$").expect("Invalid provider"));

static SUPPORTED_CURRENCIES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(USD|NGN|GBP|EUR|CAD|AUD|JPY|CHF|CNY|SEK|NZD|MXN|SGD|HKD|NOK|KRW|TRY|INR|BRL|ZAR)$",
    )
    .expect("Invalid currency")
});

#[derive(Deserialize, ToSchema, Validate)]
pub struct TopUpRequest {
    #[validate(range(
        min = 1.0,
        max = 10000.0,
        message = "Amount must be between 1 and 10,000"
    ))]
    amount: f64, // In base units of currency
    #[validate(regex(
        path = "SUPPORTED_PROVIDERS",
        message = "Provider must be 'stripe' or 'paypal'"
    ))]
    provider: String,
    #[validate(regex(path = "SUPPORTED_CURRENCIES", message = "Invalid currency"))]
    currency: String,
}

#[derive(Serialize, ToSchema)]
pub struct TopUpResponse {
    payment_id: String,
    transaction_id: String,
}

#[utoipa::path(
    post,
    path = "/api/top_up",
    request_body = TopUpRequest,
    responses(
        (status = 200, description = "Payment initiated", body = TopUpResponse),
        (status = 400, description = "Invalid input or transaction conflict"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Transaction"
)]
pub async fn top_up(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TopUpRequest>,
) -> Result<Json<TopUpResponse>, (StatusCode, String)> {
    // Validate input
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Create transaction
    let transaction_reference = Uuid::new_v4();
    let amount_cents = (req.amount * 100.0).round() as i64;

    info!("Currency: {}", req.currency.to_uppercase());

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
                description: Some(format!("{} top-up in {}", req.provider, &req.currency)),
                reference: transaction_reference,
            })
            .execute(conn)?;
        Ok(())
    })
    .map_err(|e| {
        error!("Transaction creation failed: {}", e);
        ApiError::Database(e)
    })?;

    // Initialize payment
    let client = Client::new();
    let payment_id = match req.provider.as_str() {
        "stripe" => {
            info!("Initiating {} top up with {}{}", req.provider, &req.currency, req.amount);

            let stripe_key = env::var("STRIPE_SECRET_KEY").map_err(|e| {
                error!("Stripe key not set: {}", e);
                ApiError::Payment("Stripe key not set".to_string())
            })?;
            let resp = client
                .post("https://api.stripe.com/v1/payment_intents")
                .header("Authorization", format!("Bearer {}", stripe_key))
                .form(&[
                    ("amount", amount_cents.to_string()),
                    ("currency", req.currency.to_lowercase()),
                    (
                        "metadata[transaction_id]",
                        transaction_reference.to_string(),
                    ),
                ])
                .send()
                .await
                .map_err(|e| {
                    error!("Stripe request failed: {}", e);
                    ApiError::Payment(e.to_string())
                })?;

            let status = resp.status();
            let json = resp.json::<serde_json::Value>().await.unwrap();

            if !status.is_success() {
                error!("Stripe API error: status {}, response {:?}", status, json);
                return Err(ApiError::Payment(format!(
                    "Stripe API error: {}",
                    json["error"]["message"].as_str().unwrap_or("Unknown error")
                ))
                .into());
            }

            json["client_secret"]
                .as_str()
                .ok_or_else(|| {
                    error!("Invalid Stripe response: missing client_secret");
                    ApiError::Payment("Invalid Stripe response".to_string())
                })?
                .to_string()
        }
        "paypal" => {
            info!("Initiating {} top up with {}{}", req.provider, req.currency, req.amount);

            let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|e| {
                error!("PayPal client ID not set: {}", e);
                ApiError::Payment("PayPal client ID not set".to_string())
            })?;
            let paypal_secret = std::env::var("PAYPAL_SECRET").map_err(|e| {
                error!("PayPal secret not set: {}", e);
                ApiError::Payment("PayPal secret not set".to_string())
            })?;

            let auth =
                general_purpose::STANDARD.encode(format!("{}:{}", paypal_client_id, paypal_secret));
            let resp = client
                .post("https://api-m.sandbox.paypal.com/v2/checkout/orders")
                .header("Authorization", format!("Basic {}", auth))
                .json(&serde_json::json!({
                    "intent": "CAPTURE",
                    "purchase_units": [{
                        "amount": {
                            "currency_code": req.currency.to_uppercase(),
                            "value": format!("{:.2}", req.amount),
                        },
                        "description": format!("Top-up for transaction {}", transaction_reference),
                        "custom_id": transaction_reference.to_string(),
                    }],
                    "payment_source": {
                        "paypal": {
                            "experience_context": {
                                "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
                                "return_url": "http://localhost:5173/success",
                                "cancel_url": "http://localhost:5173/cancel",
                            }
                        }
                    }
                }))
                .send()
                .await
                .map_err(|e| {
                    error!("PayPal request failed: {}", e);
                    ApiError::Payment(e.to_string())
                })?;

            let status = resp.status();
            let json = resp.json::<serde_json::Value>().await.map_err(|e| {
                error!("PayPal response parsing failed: {}", e);
                ApiError::Payment(e.to_string())
            })?;

            if !status.is_success() {
                error!("PayPal API error: status {}, response {:?}", status, json);
                return Err(ApiError::Payment(format!(
                    "PayPal API error: {}",
                    json["error"]["message"].as_str().unwrap_or("Unknown error")
                ))
                .into());
            }

            json["id"]
                .as_str()
                .ok_or_else(|| {
                    error!("Invalid PayPal response: missing order ID");
                    ApiError::Payment("Invalid PayPal response".to_string())
                })?
                .to_string()
        }
        _ => {
            error!("Invalid provider: {}", req.provider);
            return Err(ApiError::Validation(validator::ValidationErrors::new()).into());
        }
    };

    info!(
        "Top-up initiated for user {}: transaction {}, amount {}, currency {}, provider {}",
        user_id, transaction_reference, req.amount, req.currency, req.provider
    );

    Ok(Json(TopUpResponse {
        payment_id,
        transaction_id: transaction_reference.to_string(),
    }))
}
