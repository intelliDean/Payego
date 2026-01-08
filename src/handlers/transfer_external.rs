use crate::config::security_config::Claims;
use crate::models::models::{BankAccount, NewTransaction, Wallet};
use crate::schema::{bank_accounts, transactions, wallets};
use crate::{AppState, error::ApiError};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use diesel::prelude::*;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, LazyLock};
use regex::Regex;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

// Static regex for account number validation
static ACCOUNT_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{10}$").expect("Invalid account number regex")
});

#[derive(Deserialize, ToSchema)]
pub struct PayoutRequest {
    amount: f64, // Amount in NGN
    currency: String, // Currency to deduct from (e.g., "USD")
    bank_code: String,
    account_number: String,
    account_name: Option<String>,
}
#[utoipa::path(
    post,
    path = "/api/transfer/external",
    request_body = PayoutRequest,
    responses(
        (status = 200, description = "Payout initiated"),
        (status = 400, description = "Invalid bank or insufficient balance")
    ),
    security(("bearerAuth" = []))
)]
pub async fn external_transfer(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<PayoutRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    info!(
        "External transfer request: sender={}, recipient_account_number={}, amount={}, currency={}",
        claims.sub, req.account_number, req.amount, req.currency
    );

    // Validate input (account_number, bank_code)    
    if !ACCOUNT_NUMBER_RE.is_match(&req.account_number) {
        error!("Invalid account number: {}", req.account_number);
        return Err(ApiError::Payment("Account number must be 10 digits".to_string()).into());
    }

    // Validate amount
    if req.amount <= 0.0 {
        error!("Invalid amount: {}", req.amount);
        return Err(ApiError::Payment("Amount must be positive".to_string()).into());
    }
    let amount_ngn_cents = (req.amount * 100.0).round() as i64; // Amount in NGN cents

    let mut conn = state.db.get().map_err(|e: diesel::r2d2::PoolError| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Fetch sender wallet in the selected currency
    let sender_wallet = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(&req.currency))
        .first::<Wallet>(&mut conn)
        .map_err(|e: diesel::result::Error| {
            error!("Sender wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment(format!("Sender wallet not found for currency {}", req.currency))
            } else {
                ApiError::Database(e).into()
            }
        })?;

    // Fetch current exchange rate (use a tool or API)
    let exchange_rate = get_exchange_rate(&req.currency, "NGN").await.map_err(|e: (StatusCode, String)| {
        // error!("Exchange rate fetch failed: {}", e);
        ApiError::Payment("Exchange rate fetch failed".to_string())
    })?;

    let amount_to_deduct = (amount_ngn_cents as f64 / exchange_rate).round() as i64;

    // Validate balance
    if sender_wallet.balance < amount_to_deduct {
        error!(
            "Insufficient balance: available={}, required={}",
            sender_wallet.balance, amount_to_deduct
        );
        return Err(ApiError::Auth("Insufficient balance".to_string()).into());
    }


    // Create temporary Paystack recipient
    let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
        error!("PAYSTACK_SECRET_KEY not set");
        ApiError::Token("Paystack key not set".to_string())
    })?;
    let client = Client::new();
    let account_name = req.account_name.unwrap_or("External Transfer Recipient".to_string());
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
        .map_err(|e: reqwest::Error| {
            error!("Paystack recipient creation failed: {}", e);
            ApiError::Payment(format!("Paystack recipient creation failed: {}", e))
        })?;

    let status = resp.status();
    let body = resp.json::<serde_json::Value>().await.map_err(|e: reqwest::Error| {
        error!("Paystack response parsing error: {}", e);
        ApiError::Payment(format!("Paystack response error: {}", e))
    })?;

    if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
        let message = body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack error")
            .to_string();
        error!("Paystack recipient creation failed: {}", message);
        return Err(ApiError::Payment(format!("Paystack recipient creation failed: {}", message)).into());
    }


    let recipient_code = body["data"]["recipient_code"]
        .as_str()
        .ok_or_else(|| {
            error!("Invalid Paystack response: missing recipient_code");
            ApiError::Payment("Invalid Paystack response: missing recipient_code".to_string())
        })?
        .to_string();
    debug!("Paystack recipient_code: {}", recipient_code);

    // Initiate Paystack transfer
    let reference = Uuid::new_v4();
    let resp = client
        .post("https://api.paystack.co/transfer")
        .header("Authorization", format!("Bearer {}", paystack_key))
        .json(&serde_json::json!({
            "source": "balance",
            "reason": format!("External transfer from Payego in {}", req.currency),
            "amount": amount_ngn_cents,
            "recipient": recipient_code,
            "reference": reference.to_string(),
        }))
        .send()
        .await
        .map_err(|e: reqwest::Error| {
            error!("Paystack transfer failed: {}", e);
            ApiError::Payment(format!("Paystack transfer failed: {}", e))
        })?;

    let status = resp.status();
    let body = resp.json::<serde_json::Value>().await.map_err(|e: reqwest::Error| {
        error!("Paystack response parsing error: {}", e);
        ApiError::Payment(format!("Paystack response error: {}", e))
    })?;

    if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
        let message = body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack error")
            .to_string();
        error!("Paystack transfer failed: {}", message);
        return Err(ApiError::Payment(format!("Paystack transfer failed: {}", message)).into());
    }

    let transfer_code = body["data"]["transfer_code"]
        .as_str()
        .ok_or_else(|| {
            error!("Invalid Paystack response: missing transfer_code");
            ApiError::Payment("Invalid Paystack response: missing transfer_code".to_string())
        })?
        .to_string();
    debug!("Paystack transfer_code: {}", transfer_code);


    // Atomic transaction
    conn.transaction(|conn| {
        // Debit sender wallet
        diesel::update(wallets::table)
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(&req.currency))
            .set(wallets::balance.eq(wallets::balance - amount_to_deduct))
            .execute(conn)
            .map_err(|e: diesel::result::Error| {
                error!("Sender wallet update failed: {}", e);
                ApiError::Database(e)
            })?;
        info!(
            "Debited sender wallet: user_id={}, amount={}",
            user_id, amount_to_deduct
        );

        // Insert transaction
        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id,
                recipient_id: None,
                amount: -amount_to_deduct,
                transaction_type: "paystack_payout".to_string(),
                currency: req.currency.to_uppercase(),
                status: "pending".to_string(),
                provider: Some("paystack".to_string()),
                description: Some(format!("External transfer in {} to bank", &req.currency)),
                reference,
                // metadata: Some(Jsonb(json!({ "transfer_code": transfer_code, "exchange_rate": exchange_rate }))),
            })
            .execute(conn)
            .map_err(|e: diesel::result::Error| {
                error!("Payout transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        Ok::<(), ApiError>(())
    })?;
        // .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(
        "External transfer initiated: user_id={}, amount_ngn={}, currency={}, exchange_rate={}",
        user_id, req.amount, req.currency, exchange_rate
    );
    Ok(StatusCode::OK)

}





async fn get_exchange_rate(from_currency: &str, to_currency: &str) -> Result<f64, (StatusCode, String)> {
    let url = format!("https://api.exchangerate-api.com/v4/latest/{}", from_currency);
    let client = Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e: reqwest::Error| {
            error!("Exchange rate API error: {}", e);
            ApiError::Payment(format!("Exchange rate API error: {}", e))
        })?;

    let body = resp.json::<serde_json::Value>().await.map_err(|e: reqwest::Error| {
        error!("Exchange rate response parsing error: {}", e);
        ApiError::Payment(format!("Exchange rate response error: {}", e))
    })?;

    let rate = body["rates"][to_currency]
        .as_f64()
        .ok_or_else(|| {
            error!("Invalid exchange rate response");
            ApiError::Payment("Invalid exchange rate response".to_string())
        })?;

    Ok(rate)
}

