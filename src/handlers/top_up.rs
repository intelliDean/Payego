// use crate::config::security_config::Claims;
// use crate::error::ApiError;
// use crate::models::models::{AppState, NewTransaction};
// use crate::utility::get_paypal_access_token;
// use axum::{
//     extract::{Extension, Json, State},
//     http::StatusCode,
// };
// use diesel::prelude::*;
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use std::str::FromStr;
// use std::sync::Arc;
// use stripe::{
//     CheckoutSession, Client as StripeClient, CreateCheckoutSession, CreateCheckoutSessionLineItems,
//     CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData,
//     Currency,
// };
// use tracing::{error, info};
// use utoipa::ToSchema;
// use uuid::Uuid;
// use validator::{Validate, ValidationError};
//
// const SUPPORTED_PROVIDERS: &[&str] = &["stripe", "paypal"];
// const SUPPORTED_CURRENCIES: &[&str] = &[
//     "USD", "EUR", "GBP", "AUD", "BRL", "CAD", "CHF", "CNY", "HKD", "INR", "JPY", "KRW", "MXN",
//     "NGN", "NOK", "NZD", "SEK", "SGD", "TRY", "ZAR",
// ];
//
// #[derive(Deserialize, Validate, ToSchema)]
// pub struct TopUpRequest {
//     #[validate(range(
//         min = 1.0,
//         max = 10000.0,
//         message = "Amount must be between 1 and 10,000"
//     ))]
//     amount: f64,
//     #[validate(custom(function = "validate_provider"))]
//     provider: String,
//     #[validate(custom(function = "validate_currency"))]
//     currency: String,
// }
//
// #[derive(Serialize, ToSchema)]
// pub struct TopUpResponse {
//     session_url: Option<String>, // For Stripe Checkout
//     payment_id: Option<String>,  // For PayPal order ID
//     transaction_id: String,
//     amount: f64,
// }
//
// fn validate_provider(provider: &str) -> Result<(), ValidationError> {
//     if SUPPORTED_PROVIDERS.contains(&provider) {
//         Ok(())
//     } else {
//         Err(ValidationError::new(
//             "Provider must be 'stripe' or 'paypal'",
//         ))
//     }
// }
//
// fn validate_currency(currency: &str) -> Result<(), ValidationError> {
//     if SUPPORTED_CURRENCIES.contains(&currency) {
//         Ok(())
//     } else {
//         Err(ValidationError::new("Invalid currency"))
//     }
// }
//
// #[utoipa::path(
//     post,
//     path = "/api/top_up",
//     request_body = TopUpRequest,
//     responses(
//         (status = 200, description = "Payment initiated", body = TopUpResponse),
//         (status = 400, description = "Invalid input"),
//         (status = 401, description = "Unauthorized"),
//         (status = 500, description = "Internal server error")
//     ),
//     security(("bearerAuth" = [])),
//     tag = "Transaction"
// )]
// pub async fn top_up(
//     State(state): State<Arc<AppState>>,
//     Extension(claims): Extension<Claims>,
//     Json(req): Json<TopUpRequest>,
// ) -> Result<Json<TopUpResponse>, (StatusCode, String)> {
//     req.validate().map_err(|e| {
//         error!("Validation error: {}", e);
//         ApiError::Validation(e)
//     })?;
//
//     let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
//         error!("Invalid user ID: {}", e);
//         ApiError::Auth("Invalid user ID".to_string())
//     })?;
//
//     let conn = &mut state.db.get().map_err(|e| {
//         error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     let transaction_id = Uuid::new_v4();
//     let amount_cents = (req.amount * 100.0).round() as i64;
//
//     conn.transaction(|conn| {
//         diesel::insert_into(crate::schema::transactions::table)
//             .values(NewTransaction {
//                 user_id,
//                 recipient_id: None,
//                 amount: amount_cents,
//                 transaction_type: format!("topup_{}", req.provider),
//                 currency: req.currency.to_uppercase(),
//                 status: "pending".to_string(),
//                 provider: Some(req.provider.clone()),
//                 description: Some(format!("{} top-up in {}", req.provider, req.currency)),
//                 reference: transaction_id,
//             })
//             .execute(conn)?;
//         Ok(())
//     })
//     .map_err(|e| {
//         error!("Transaction creation failed: {}", e);
//         ApiError::Database(e)
//     })?;
//
//     let client = Client::new();
//     let payment_id = match req.provider.as_str() {
//         "stripe" => {
//             info!("Initiating Stripe top-up: {} {}", req.amount, req.currency);
//             let stripe_client = StripeClient::new(&state.stripe_secret_key);
//             let line_item = CreateCheckoutSessionLineItems {
//                 quantity: Some(1),
//                 price_data: Some(CreateCheckoutSessionLineItemsPriceData {
//                     currency: Currency::from_str(&req.currency.to_lowercase()).map_err(|e| {
//                         error!("Invalid currency: {}", req.currency.to_lowercase());
//                         ApiError::Auth("Invalid currency".to_string())
//                     })?,
//                     unit_amount: Some(amount_cents),
//                     product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
//                         name: "Account Top-Up".to_string(),
//                         description: Some(format!(
//                             "Add {} {} to your account",
//                             req.amount, req.currency
//                         )),
//                         ..Default::default()
//                     }),
//                     ..Default::default()
//                 }),
//                 ..Default::default()
//             };
//
//             let mut metadata = std::collections::HashMap::new();
//             metadata.insert("transaction_id".to_string(), transaction_id.to_string());
//
//             let session_params = CreateCheckoutSession {
//                 success_url: Some(&format!(
//                     "http://localhost:5173/success?transaction_id={}",
//                     transaction_id
//                 )),
//                 cancel_url: Some(&format!("http://localhost:5173/top-up")),
//                 mode: Some(stripe::CheckoutSessionMode::Payment),
//                 line_items: Some(vec![line_item]),
//                 metadata: Some(metadata),
//                 ..Default::default()
//             };
//
//             let session = CheckoutSession::create(&stripe_client, session_params)
//                 .await
//                 .map_err(|e| {
//                     error!("Stripe Checkout session creation failed: {}", e);
//                     ApiError::Payment(format!("Stripe error: {}", e))
//                 })?;
//
//             session.url.ok_or_else(|| {
//                 error!("Stripe session created but no URL returned");
//                 ApiError::Payment("Failed to create payment session".to_string())
//             })
//         }
//         "paypal" => {
//             info!("Initiating PayPal top-up: {} {}", req.amount, req.currency);
//             let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|e| {
//                 error!("PayPal client ID not set: {}", e);
//                 ApiError::Payment("PayPal client ID not set".to_string())
//             })?;
//             let paypal_secret = std::env::var("PAYPAL_SECRET").map_err(|e| {
//                 error!("PayPal secret not set: {}", e);
//                 ApiError::Payment("PayPal secret not set".to_string())
//             })?;
//
//             let paypal_access_token =
//                 get_paypal_access_token(&client, &paypal_client_id, &paypal_secret)
//                     .await
//                     .map_err(|e| {
//                         // error!("Failed to get PayPal access token: {}", e);
//                         e
//                     })?;
//
//             info!("Access Token: {:?}", paypal_access_token);
//
//             let resp = client
//                 .post("https://api-m.sandbox.paypal.com/v2/checkout/orders")
//                 .bearer_auth(paypal_access_token)
//                 .json(&serde_json::json!({
//                     "intent": "CAPTURE",
//                     "purchase_units": [{
//                         "amount": {
//                             "currency_code": req.currency.to_uppercase(),
//                             "value": format!("{:.2}", req.amount),
//                         },
//                         "description": format!("Top-up for transaction {}", transaction_id),
//                         "custom_id": transaction_id.to_string(),
//                     }],
//                     "application_context": {
//                         "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
//                         "return_url": format!("{}/success?transaction_id={}", state.app_url, transaction_id),
//                         "cancel_url": format!("{}/top-up", state.app_url),
//                         "brand_name": "Payego",
//                         "user_action": "PAY_NOW"
//                     }
//                 }))
//                 .send()
//                 .await
//                 .map_err(|e| {
//                     error!("PayPal request failed: {}", e);
//                     ApiError::Payment(format!("PayPal error: {}", e))
//                 })?;
//
//             let status = resp.status();
//
//             let json = resp.json::<serde_json::Value>().await.map_err(|e| {
//                 error!("PayPal response parsing failed: {}", e);
//                 ApiError::Payment(format!("PayPal error: {}", e))
//             })?;
//
//             if !status.is_success() {
//                 error!("PayPal API error: status {}, response {:?}", status, json);
//                 return Err(ApiError::Payment(format!(
//                     "PayPal API error: {}",
//                     json["error_description"]
//                         .as_str()
//                         .unwrap_or("Unknown error")
//                 ))
//                 .into());
//             }
//
//             Ok(json["id"]
//                 .as_str()
//                 .ok_or_else(|| {
//                     error!("Invalid PayPal response: missing order ID");
//                     ApiError::Payment("Invalid PayPal response".to_string())
//                 })?
//                 .to_string())
//         }
//         _ => Err(ApiError::Auth("Invalid provider".to_string()).into()),
//     }?;
//
//     info!(
//         "Top-up initiated for user {}: transaction {}, amount {}, currency {}, provider {}",
//         user_id, transaction_id, req.amount, req.currency, req.provider
//     );
//
//     Ok(Json(TopUpResponse {
//         session_url: if req.provider == "stripe" {
//             Some(payment_id.clone())
//         } else {
//             None
//         },
//         payment_id: if req.provider == "paypal" {
//             Some(payment_id)
//         } else {
//             None
//         },
//         transaction_id: transaction_id.to_string(),
//         amount: req.amount,
//     }))
// }



//===============

use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::models::models::{AppState, NewTransaction};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use diesel::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use stripe::{CheckoutSession, Client as StripeClient, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData, Currency};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

const SUPPORTED_PROVIDERS: &[&str] = &["stripe", "paypal"];
const SUPPORTED_CURRENCIES: &[&str] = &[
    "USD", "EUR", "GBP", "AUD", "BRL", "CAD", "CHF", "CNY", "HKD", "INR", "JPY", "KRW", "MXN",
    "NGN", "NOK", "NZD", "SEK", "SGD", "TRY", "ZAR",
];

#[derive(Deserialize, Validate, ToSchema)]
pub struct TopUpRequest {
    #[validate(range(
        min = 1.0,
        max = 10000.0,
        message = "Amount must be between 1 and 10,000"
    ))]
    amount: f64,
    #[validate(custom(function = "validate_provider"))]
    provider: String,
    #[validate(custom(function = "validate_currency"))]
    currency: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TopUpResponse {
    session_url: Option<String>,
    payment_id: Option<String>,
    transaction_id: String,
    amount: f64,
}

fn validate_provider(provider: &str) -> Result<(), ValidationError> {
    if SUPPORTED_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err(ValidationError::new("Provider must be 'stripe' or 'paypal'"))
    }
}

fn validate_currency(currency: &str) -> Result<(), ValidationError> {
    if SUPPORTED_CURRENCIES.contains(&currency) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid currency"))
    }
}

async fn get_paypal_access_token(
    client: &Client,
    client_id: &str,
    secret: &str,
) -> Result<String, ApiError> {
    let response = client
        .post("https://api-m.sandbox.paypal.com/v1/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .basic_auth(client_id, Some(secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to get PayPal access token: {}", e);
            ApiError::Payment(format!("PayPal auth error: {}", e))
        })?;

    let json = response.json::<serde_json::Value>().await.map_err(|e| {
        error!("PayPal token response parsing failed: {}", e);
        ApiError::Payment(format!("PayPal auth error: {}", e))
    })?;

    json["access_token"]
        .as_str()
        .ok_or_else(|| {
            error!("Invalid PayPal token response: missing access_token");
            ApiError::Payment("Invalid PayPal token response".to_string())
        })
        .map(|s| s.to_string())
}

#[utoipa::path(
    post,
    path = "/api/top_up",
    request_body = TopUpRequest,
    responses(
        (status = 200, description = "Payment initiated", body = TopUpResponse),
        (status = 400, description = "Invalid input"),
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
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    info!("APP URL: {}", &state.app_url);

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let transaction_id = Uuid::new_v4();
    let amount_cents = (req.amount * 100.0).round() as i64;

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
            })
            .execute(conn)?;

        Ok::<(), ApiError>(())

    })?;

    let client = Client::new();
    let (session_url, payment_id) = match req.provider.as_str() {
        "stripe" => {
            info!("Initiating Stripe top-up: {} {}", req.amount, req.currency);
            let stripe_client = StripeClient::new(&state.stripe_secret_key);
            let line_item = CreateCheckoutSessionLineItems {
                quantity: Some(1),
                price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                    currency: Currency::from_str(&req.currency.to_lowercase()).map_err(|e| {
                        error!("Invalid currency: {}", req.currency.to_lowercase());
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
                mode: Some(stripe::CheckoutSessionMode::Payment),
                line_items: Some(vec![line_item]),
                metadata: Some(metadata),
                ..Default::default()
            };

            let session = CheckoutSession::create(&stripe_client, session_params)
                .await
                .map_err(|e| {
                    error!("Stripe Checkout session creation failed: {}", e);
                    ApiError::Payment(format!("Stripe error: {}", e))
                })?;

            (session.url, None)
        }
        "paypal" => {
            info!("Initiating PayPal top-up: {} {}", req.amount, req.currency);
            let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|e| {
                error!("PayPal client ID not set: {}", e);
                ApiError::Payment("PayPal client ID not set".to_string())
            })?;
            let paypal_secret = std::env::var("PAYPAL_SECRET").map_err(|e| {
                error!("PayPal secret not set: {}", e);
                ApiError::Payment("PayPal secret not set".to_string())
            })?;

            let paypal_access_token = get_paypal_access_token(&client, &paypal_client_id, &paypal_secret)
                .await
                .map_err(|e| {
                    // error!("Failed to get PayPal access token: {}", e);
                    e
                })?;

            info!("PayPal Access Token: {}", paypal_access_token);

            let resp = client
                .post("https://api-m.sandbox.paypal.com/v2/checkout/orders")
                .bearer_auth(paypal_access_token)
                .json(&serde_json::json!({
                    "intent": "CAPTURE",
                    "purchase_units": [{
                        "amount": {
                            "currency_code": req.currency.to_uppercase(),
                            "value": format!("{:.2}", req.amount),
                        },
                        "description": format!("Top-up for transaction {}", transaction_id),
                        "custom_id": transaction_id.to_string(),
                    }],
                    "application_context": {
                        "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
                        "return_url": format!("{}/success?transaction_id={}", state.app_url, transaction_id),
                        "cancel_url": format!("{}/top-up", state.app_url),
                        "brand_name": "Payego",
                        "user_action": "PAY_NOW"
                    }
                }))
                .send()
                .await
                .map_err(|e| {
                    error!("PayPal request failed: {}", e);
                    ApiError::Payment(format!("PayPal error: {}", e))
                })?;

            let status = resp.status();
            let json = resp.json::<serde_json::Value>().await.map_err(|e| {
                error!("PayPal response parsing failed: {}", e);
                ApiError::Payment(format!("PayPal error: {}", e))
            })?;

            if !status.is_success() {
                error!("PayPal API error: status {}, response {:?}", status, json);
                return Err(ApiError::Payment(format!(
                    "PayPal API error: {}",
                    json["error_description"].as_str().unwrap_or("Unknown error")
                )).into());
            }

            let payment_id = json["id"].as_str().ok_or_else(|| {
                error!("Invalid PayPal response: missing order ID");
                ApiError::Payment("Invalid PayPal response".to_string())
            })?;

            (None, Some(payment_id.to_string()))
        }
        _ => {
            error!("Invalid provider: {}", req.provider);
            return Err(ApiError::Validation(validator::ValidationErrors::new()).into());
        }
    };

    info!(
        "Top-up initiated for user {}: transaction {}, amount {}, currency {}, provider {}",
        user_id, transaction_id, req.amount, req.currency, req.provider
    );

    Ok(Json(TopUpResponse {
        session_url,
        payment_id,
        transaction_id: transaction_id.to_string(),
        amount: req.amount,
    }))
}
