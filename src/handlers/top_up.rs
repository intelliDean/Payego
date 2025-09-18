use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::models::user_models::{AppState, NewTransaction};
use crate::schema::users::{email, username};
use axum::{Extension, Json, extract::State, http::StatusCode};
use diesel::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, LazyLock};
use regex::Regex;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use jsonwebtoken::errors::ErrorKind::Base64;
use validator::{Validate, ValidationError};


static SUPPORTED_PROVIDERS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(stripe|paypal)$").expect("Invalid provider")
});

#[derive(Deserialize, ToSchema, Validate)]
pub struct TopUpRequest {
    #[validate(range(
        min = 1.0,
        max = 10000.0,
        message = "Amount must be between 1 and 10,000 USD"
    ))]
    amount: f64, // In dollars
    #[validate(regex(
        path = "SUPPORTED_PROVIDERS",
        message = "Provider must be 'stripe' or 'paypal'"
    ))]
    provider: String,
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

    conn.transaction(|conn| {
        diesel::insert_into(crate::schema::transactions::table)
            .values(NewTransaction {
                user_id,
                recipient_id: None,
                amount: amount_cents,
                transaction_type: format!("topup_{}", req.provider),
                status: "pending".to_string(),
                provider: Some(req.provider.clone()),
                description: Some(format!("{} top-up", req.provider)),
                reference: transaction_reference, // Use Uuid directly
            })
            .execute(conn)?;
        Ok(())
    }).map_err(|e| {
        error!("Transaction creation failed: {}", e);
        ApiError::Database(e)
    })?;

    // Initialize payment
    let client = Client::new();
    let payment_id = match req.provider.as_str() {
        "stripe" => {
            let stripe_key = env::var("STRIPE_SECRET_KEY").map_err(|e| {
                error!("Stripe key not set: {}", e);
                ApiError::Payment("Stripe key not set".to_string())
            })?;
            let resp = client
                .post("https://api.stripe.com/v1/payment_intents")
                .header("Authorization", format!("Bearer {}", stripe_key))
                .form(&[
                    ("amount", amount_cents.to_string()),
                    ("currency", "usd".to_string()),
                    (
                        "metadata[transaction_id]",
                        transaction_reference.to_string(),
                    ),
                    // ("return_url", "http://localhost:5173/success".to_string()),
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
                )).into());
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
            let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|e| {
                error!("PayPal client ID not set: {}", e);
                ApiError::Payment("PayPal client ID not set".to_string())
            })?;
            let paypal_secret = std::env::var("PAYPAL_SECRET").map_err(|e| {
                error!("PayPal secret not set: {}", e);
                ApiError::Payment("PayPal secret not set".to_string())
            })?;

            let auth = general_purpose::STANDARD.encode(format!("{}:{}", paypal_client_id, paypal_secret));
            let resp = client
                .post("https://api-m.sandbox.paypal.com/v2/checkout/orders")
                .header("Authorization", format!("Basic {}", auth))
                .json(&serde_json::json!({
                    "intent": "CAPTURE",
                    "purchase_units": [{
                        "amount": {
                            "currency_code": "USD",
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
                )).into());
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
            return Err(<(StatusCode, String)>::from(ApiError::Validation(
                validator::ValidationErrors::new(),
            )));
        }
    };

    info!(
        "Top-up initiated for user {}: transaction {}, amount ${:.2}, provider {}",
        user_id, transaction_reference, req.amount, req.provider
    );

    Ok(Json(TopUpResponse {
        payment_id,
        transaction_id: transaction_reference.to_string(),
    }))
}






//==================================================

// 
// 
// use axum::{extract::State, http::StatusCode, Json, Extension};
// use diesel::prelude::*;
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use std::sync::{Arc, LazyLock};
// use regex::Regex;
// use tracing::{error, info};
// use utoipa::ToSchema;
// use validator::{Validate, ValidationError};
// use uuid::Uuid;
// use base64::engine::general_purpose::STANDARD as Base64;
// use jsonwebtoken::errors::ErrorKind::Base64;
// use crate::config::security_config::Claims;
// use crate::models::user_models::{AppState, NewTransaction};
// 
// // ApiError enum (aligned with register, login, and auth_middleware)
// #[derive(Debug)]
// pub enum ApiError {
//     Database(diesel::result::Error),
//     Bcrypt(bcrypt::BcryptError),
//     Validation(validator::ValidationErrors),
//     Auth(String),
//     Token(String),
//     Payment(String),
//     DatabaseConnection(String),
// }
// 
// impl From<diesel::result::Error> for ApiError {
//     fn from(err: diesel::result::Error) -> Self {
//         ApiError::Database(err)
//     }
// }
// 
// impl From<bcrypt::BcryptError> for ApiError {
//     fn from(err: bcrypt::BcryptError) -> Self {
//         ApiError::Bcrypt(err)
//     }
// }
// 
// impl From<validator::ValidationErrors> for ApiError {
//     fn from(err: validator::ValidationErrors) -> Self {
//         ApiError::Validation(err)
//     }
// }
// 
// impl From<reqwest::Error> for ApiError {
//     fn from(err: reqwest::Error) -> Self {
//         ApiError::Payment(err.to_string())
//     }
// }
// 
// impl From<ApiError> for (StatusCode, String) {
//     fn from(err: ApiError) -> Self {
//         match err {
//             ApiError::Database(e) => match e {
//                 diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => (
//                     StatusCode::BAD_REQUEST,
//                     "Transaction ID conflict".to_string(),
//                 ),
//                 _ => (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     format!("Database error: {}", e),
//                 ),
//             },
//             ApiError::Bcrypt(_) => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Password verification error".to_string(),
//             ),
//             ApiError::Validation(errors) => (
//                 StatusCode::BAD_REQUEST,
//                 format!("Validation error: {}", errors),
//             ),
//             ApiError::Auth(msg) => (
//                 StatusCode::UNAUTHORIZED,
//                 msg,
//             ),
//             ApiError::Token(msg) => (
//                 StatusCode::UNAUTHORIZED,
//                 format!("Invalid token: {}", msg),
//             ),
//             ApiError::Payment(msg) => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Payment provider error: {}", msg),
//             ),
//             ApiError::DatabaseConnection(msg) => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Database connection error: {}", msg),
//             ),
//         }
//     }
// }
// 
// static SUPPORTED_PROVIDERS: LazyLock<Regex> = LazyLock::new(|| {
//     Regex::new(r"^(stripe|paypal)$").expect("Invalid regex pattern")
// });
// 
// #[derive(Deserialize, ToSchema, Validate)]
// pub struct TopUpRequest {
//     #[validate(range(
//         min = 1.0,
//         max = 10000.0,
//         message = "Amount must be between 1 and 10,000 USD"
//     ))]
//     amount: f64, // In dollars
//     #[validate(regex(
//         path = "SUPPORTED_PROVIDERS",
//         message = "Provider must be 'stripe' or 'paypal'"
//     ))]
//     provider: String,
// }
// 
// #[derive(Serialize, ToSchema)]
// pub struct TopUpResponse {
//     payment_id: String, // Renamed from client_secret for clarity
//     transaction_id: Uuid,
// }
// 
// #[utoipa::path(
//     post,
//     path = "/api/top_up",
//     request_body = TopUpRequest,
//     responses(
//         (status = 200, description = "Payment initiated", body = TopUpResponse),
//         (status = 400, description = "Invalid input or transaction conflict"),
//         (status = 401, description = "Unauthorized"),
//         (status = 500, description = "Internal server error")
//     ),
//     security(("Bearer" = [])),
//     tag = "Transaction"
// )]
// pub async fn top_up(
//     State(state): State<Arc<AppState>>,
//     Extension(claims): Extension<Claims>,
//     Json(req): Json<TopUpRequest>,
// ) -> Result<Json<TopUpResponse>, (StatusCode, String)> {
//     // Validate input
//     req.validate().map_err(|e| {
//         error!("Validation error: {}", e);
//         ApiError::Validation(e)
//     })?;
// 
//     // Parse user ID
//     let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
//         error!("Invalid user ID: {}", e);
//         ApiError::Auth("Invalid user ID".to_string())
//     })?;
// 
//     // Get database connection
//     let mut conn = state.db.get().map_err(|e| {
//         error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
// 
//     // Create transaction
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
//                 status: "pending".to_string(),
//                 provider: Some(req.provider.clone()),
//                 description: Some(format!("{} top-up", req.provider)),
//                 reference: transaction_id, // Use Uuid directly
//             })
//             .execute(conn)?;
//         Ok(())
//     }).map_err(|e| {
//         error!("Transaction creation failed: {}", e);
//         ApiError::Database(e)
//     })?;
// 
//     // Initialize payment
//     let client = Client::new();
//     let payment_id = match req.provider.as_str() {
//         "stripe" => {
//             let stripe_key = std::env::var("STRIPE_SECRET_KEY").map_err(|e| {
//                 error!("Stripe key not set: {}", e);
//                 ApiError::Payment("Stripe key not set".to_string())
//             })?;
//             let resp = client
//                 .post("https://api.stripe.com/v1/payment_intents")
//                 .header("Authorization", format!("Bearer {}", stripe_key))
//                 .form(&[
//                     ("amount", amount_cents.to_string()),
//                     ("currency", "usd".to_string()),
//                     ("metadata[transaction_id]", transaction_id.to_string()),
//                 ])
//                 .send()
//                 .await
//                 .map_err(|e| {
//                     error!("Stripe request failed: {}", e);
//                     ApiError::Payment(e.to_string())
//                 })?;
// 
//             let status = resp.status();
//             let json = resp.json::<serde_json::Value>().await.map_err(|e| {
//                 error!("Stripe response parsing failed: {}", e);
//                 ApiError::Payment(e.to_string())
//             })?;
// 
//             if !status.is_success() {
//                 error!("Stripe API error: status {}, response {:?}", status, json);
//                 return Err(ApiError::Payment(format!(
//                     "Stripe API error: {}",
//                     json["error"]["message"].as_str().unwrap_or("Unknown error")
//                 )).into());
//             }
// 
//             json["client_secret"].as_str()
//                 .ok_or_else(|| {
//                     error!("Invalid Stripe response: missing client_secret");
//                     ApiError::Payment("Invalid Stripe response".to_string())
//                 })?
//                 .to_string()
//         }
//         "paypal" => {
//             let paypal_client_id = std::env::var("PAYPAL_CLIENT_ID").map_err(|e| {
//                 error!("PayPal client ID not set: {}", e);
//                 ApiError::Payment("PayPal client ID not set".to_string())
//             })?;
//             let paypal_secret = std::env::var("PAYPAL_SECRET").map_err(|e| {
//                 error!("PayPal secret not set: {}", e);
//                 ApiError::Payment("PayPal secret not set".to_string())
//             })?;
// 
//             let auth = Base64::encode(format!("{}:{}", paypal_client_id, paypal_secret));
//             let resp = client
//                 .post("https://api-m.sandbox.paypal.com/v2/checkout/orders")
//                 .header("Authorization", format!("Basic {}", auth))
//                 .json(&serde_json::json!({
//                     "intent": "CAPTURE",
//                     "purchase_units": [{
//                         "amount": {
//                             "currency_code": "USD",
//                             "value": format!("{:.2}", req.amount),
//                         },
//                         "description": format!("Top-up for transaction {}", transaction_id),
//                         "custom_id": transaction_id.to_string(),
//                     }],
//                     "payment_source": {
//                         "paypal": {
//                             "experience_context": {
//                                 "payment_method_preference": "IMMEDIATE_PAYMENT_REQUIRED",
//                                 "return_url": "https://your-app.com/success",
//                                 "cancel_url": "https://your-app.com/cancel",
//                             }
//                         }
//                     }
//                 }))
//                 .send()
//                 .await
//                 .map_err(|e| {
//                     error!("PayPal request failed: {}", e);
//                     ApiError::Payment(e.to_string())
//                 })?;
// 
//             let status = resp.status();
//             let json = resp.json::<serde_json::Value>().await.map_err(|e| {
//                 error!("PayPal response parsing failed: {}", e);
//                 ApiError::Payment(e.to_string())
//             })?;
// 
//             if !status.is_success() {
//                 error!("PayPal API error: status {}, response {:?}", status, json);
//                 return Err(ApiError::Payment(format!(
//                     "PayPal API error: {}",
//                     json["error"]["message"].as_str().unwrap_or("Unknown error")
//                 )));
//             }
// 
//             json["id"].as_str()
//                 .ok_or_else(|| {
//                     error!("Invalid PayPal response: missing order ID");
//                     ApiError::Payment("Invalid PayPal response".to_string())
//                 })?
//                 .to_string()
//         }
//         _ => {
//             error!("Invalid provider: {}", req.provider);
//             return Err(ApiError::Validation(validator::ValidationErrors::new()));
//         }
//     };
// 
//     info!(
//         "Top-up initiated for user {}: transaction {}, amount ${:.2}, provider {}",
//         user_id, transaction_id, req.amount, req.provider
//     );
// 
//     Ok(Json(TopUpResponse {
//         payment_id,
//         transaction_id,
//     }))
// }