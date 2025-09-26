use crate::config::security_config::Claims;
use crate::models::models::NewBankAccount;
use crate::schema::{bank_accounts, wallets};
use crate::{error::ApiError, AppState};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
pub struct BankRequest {
    #[validate(length(min = 1, message = "Bank code is required"))]
    bank_code: String,
    #[validate(regex(
        path = "ACCOUNT_NUMBER_RE",
        message = "Account number must be 10 digits"
    ))]
    account_number: String,
    #[validate(length(min = 1, message = "Account name is required"))]
    bank_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct BankResponse {
    transaction_id: String,
    account_name: String,
}

static ACCOUNT_NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{10}$").expect("Invalid account number regex"));

#[utoipa::path(
    post,
    path = "/api/add_bank",
    request_body = BankRequest,
    responses(
        (status = 201, description = "Bank account added", body = BankResponse),
        (status = 400, description = "Invalid bank details"),
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
) -> Result<Json<BankResponse>, (StatusCode, String)> {
    info!(
        "Add bank request: user_id = {}, bank_code = {}, account_number = {}",
        claims.sub, req.bank_code, req.account_number
    );

    // Validate input
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Parse user_id
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Create or verify NGN wallet
    diesel::insert_into(wallets::table)
        .values((
            wallets::user_id.eq(user_id),
            wallets::balance.eq(0),
            wallets::currency.eq("NGN"),
        ))
        .on_conflict((wallets::user_id, wallets::currency))
        .do_nothing()
        .execute(conn)
        .map_err(|e| {
            error!("Wallet creation failed: {}", e);
            ApiError::Database(e)
        })?;

    // Check for duplicate bank account
    let existing = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .filter(bank_accounts::bank_code.eq(&req.bank_code))
        .filter(bank_accounts::account_number.eq(&req.account_number))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            error!("Bank account lookup failed: {}", e);
            ApiError::Database(e)
        })?;
    if existing {
        error!(
            "Bank account already exists: user_id={}, bank_code={}, account_number={}",
            user_id, req.bank_code, req.account_number
        );
        return Err((
            StatusCode::CONFLICT,
            "Bank account already exists".to_string(),
        ));
    }

    // Verify bank account with Paystack (resolve account name)
    let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
        error!("PAYSTACK_SECRET_KEY not set");
        ApiError::Payment("Paystack key not set".to_string())
    })?;
    let client = Client::new();
    let resolve_resp = client
        .get(format!(
            "https://api.paystack.co/bank/resolve?account_number={}&bank_code={}",
            req.account_number, req.bank_code
        ))
        .header("Authorization", format!("Bearer {}", paystack_key))
        .send()
        .await
        .map_err(|e| {
            error!("Paystack resolve API error: {}", e);
            ApiError::Payment(format!("Paystack resolve API error: {}", e))
        })?;

    let resolve_status = resolve_resp.status();
    let resolve_body = resolve_resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            error!("Paystack resolve response parsing error: {}", e);
            ApiError::Payment("Paystack resolve response error".to_string())
        })?;

    if !resolve_status.is_success() || resolve_body["status"].as_bool().unwrap_or(false) == false {
        let message = resolve_body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack resolve error")
            .to_string();
        error!("Paystack account resolution failed: {}", message);
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Paystack account resolution failed: {}", message),
        ));
    }

    let account_name = resolve_body["data"]["account_name"]
        .as_str()
        .ok_or_else(|| {
            error!("Missing account_name in Paystack resolve response");
            ApiError::Payment("Invalid Paystack resolve response: missing account_name".to_string())
        })?
        .to_string();
    debug!("Resolved account_name: {}", account_name);

    // Create Paystack transfer recipient
    let recipient_resp = client
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
            error!("Paystack transferrecipient API error: {}", e);
            ApiError::Payment(format!("Paystack transferrecipient API error: {}", e))
        })?;

    let recipient_status = recipient_resp.status();
    let recipient_body = recipient_resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            error!("Paystack transferrecipient response parsing error: {}", e);
            ApiError::Payment("Paystack transferrecipient response error".to_string())
        })?;

    if !recipient_status.is_success()
        || recipient_body["status"].as_bool().unwrap_or(false) == false
    {
        let message = recipient_body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack transferrecipient error")
            .to_string();
        error!("Paystack transferrecipient creation failed: {}", message);
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Paystack transferrecipient creation failed: {}", message),
        ));
    }

    let recipient_code = recipient_body["data"]["recipient_code"]
        .as_str()
        .ok_or_else(|| {
            error!("Missing recipient_code in Paystack transferrecipient response");
            ApiError::Payment("Invalid Paystack response: missing recipient_code".to_string())
        })?
        .to_string();
    debug!("Paystack recipient_code: {}", recipient_code);

    // Insert bank account
    let bank_account_id = Uuid::new_v4();
    diesel::insert_into(bank_accounts::table)
        .values(NewBankAccount {
            id: bank_account_id,
            user_id,
            bank_code: req.bank_code,
            account_number: req.account_number,
            account_name: Some(account_name.clone()),
            bank_name: Some(req.bank_name),
            paystack_recipient_code: Some(recipient_code.clone()),
            is_verified: true,
        })
        .execute(conn)
        .map_err(|e| {
            error!("Failed to add bank account: {}", e);
            if e.to_string().contains("unique") {
                ApiError::Payment("Bank account already exists".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    info!(
        "Bank account added: user_id = {}, recipient_code = {}, account_name = {}",
        user_id, recipient_code, account_name
    );
    Ok(Json(BankResponse {
        transaction_id: bank_account_id.to_string(),
        account_name,
    }))
}
