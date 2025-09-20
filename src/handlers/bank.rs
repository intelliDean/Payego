use crate::config::security_config::Claims;
use crate::models::user_models::{NewBankAccount, Wallet};
use crate::schema::{bank_accounts, wallets};
use crate::{AppState, error::ApiError};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use diesel::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/add_bank",
    request_body = BankRequest,
    responses(
        (status = 201, description = "Bank account added"),
        (status = 400, description = "Invalid bank details or currency mismatch"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Bank account already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Payment"
)]
pub async fn add_bank_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<BankRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!(
        "Add bank request: user_id = {}, bank_code = {}, account_number = {}",
        claims.sub, req.bank_code, req.account_number
    );

    // Validate input
    req.validate()?;

    let account_name = req.account_name.unwrap_or("User Bank".to_string());
    if account_name.trim().is_empty() {
        error!("Account name is empty");
        return Err(ApiError::Auth("Account name cannot be empty".to_string()).into());
    }
    debug!(
        "Validated input: bank_code={}, account_number = {}, account_name = {}",
        req.bank_code, req.account_number, account_name
    );

    // Parse user_id
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Get database connection
    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Verify wallet currency is NGN
    let wallet = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .first::<Wallet>(&mut conn)
        .map_err(|e| {
            error!("Wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Wallet not found".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;
    if wallet.currency != "NGN" {
        error!("Wallet currency is {}, expected NGN", wallet.currency);
        return Err(ApiError::Auth("Wallet currency must be NGN for Paystack".to_string()).into());
    }

    // Check for duplicate bank account
    let existing = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .select(diesel::dsl::count_star())
        .first::<i64>(&mut conn)
        .map(|count| count > 0)
        .map_err(|e| {
            error!("Bank account lookup failed: {}", e);
            ApiError::Database(e)
        })?;
    if existing {
        error!("Bank account already exists for user_id={}", user_id);
        return Err(ApiError::Auth("Bank account already exists".to_string()).into());
    }

    // Verify bank account with Paystack
    let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
        error!("PAYSTACK_SECRET_KEY not set");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Paystack key not set".to_string(),
        )
    })?;
    let client = Client::new();
    let resp = client
        .post("https://api.paystack.co/transferrecipient")
        .header("Authorization", format!("Bearer {}", paystack_key))
        .json(&serde_json::json!({
            "type": "nuban",
            "name": account_name,
            "account_number": req.account_number,
            "bank_code": req.bank_code,
            "currency": "NGN"
        }))
        .send()
        .await
        .map_err(|e| {
            error!("Paystack API error: {}", e);
            ApiError::Payment(format!("Paystack API error: {}", e))
        })?;

    let status = resp.status();
    let body = resp.json::<serde_json::Value>().await.map_err(|e| {
        error!("Paystack response parsing error: {}", e);
        ApiError::Payment("Paystack response error".to_string())
    })?;

    if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
        let message = body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack error")
            .to_string();
        error!("Paystack verification failed: {}", message);
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Paystack verification failed: {}", message),
        ));
    }

    let recipient_code = body["data"]["recipient_code"]
        .as_str()
        .ok_or(ApiError::Payment(
            "Invalid Paystack response: missing recipient_code".to_string(),
        ))?
        .to_string();
    debug!("Paystack recipient_code: {}", recipient_code);

    // Insert bank account
    diesel::insert_into(bank_accounts::table)
        .values(NewBankAccount {
            id: Uuid::new_v4(),
            user_id,
            bank_code: req.bank_code,
            account_number: req.account_number,
            account_name: Some(account_name),
            paystack_recipient_code: Some(recipient_code.clone()),
            is_verified: true,
        })
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to add bank account: {}", e);
            if e.to_string().contains("unique") {
                ApiError::Payment("Bank account already exists".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    info!(
        "Bank account added: user_id = {}, recipient_code = {}",
        user_id, recipient_code
    );
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, ToSchema)]
pub struct BankRequest {
    bank_code: String,
    account_number: String,
    account_name: Option<String>,
}

impl BankRequest {
    pub fn validate(&self) -> Result<(), (StatusCode, String)> {
        let bank_code_re = Regex::new(r"^\d{3,5}$").unwrap();
        let account_number_re = Regex::new(r"^\d{10}$").unwrap();

        if !bank_code_re.is_match(&self.bank_code) {
            return Err(ApiError::Auth("Bank code must be 3-5 digits".to_string()).into());
        }

        if !account_number_re.is_match(&self.account_number) {
            return Err(ApiError::Auth("Account number must be 10 digits".to_string()).into());
        }

        Ok(())
    }
}
