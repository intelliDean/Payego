use axum::{
    extract::{State, Extension},
    Json,
    http::StatusCode,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use regex::Regex;
use reqwest::Client;
use uuid::Uuid;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use crate::{AppState, error::ApiError};
use crate::config::security_config::Claims;
use crate::schema::{bank_accounts, transactions, wallets};
use crate::models::models::{BankAccount, NewTransaction, Wallet};

#[derive(Deserialize, ToSchema)]
pub struct WithdrawRequest {
    pub amount: f64, // Amount in the selected currency
    pub currency: String, // Currency to withdraw from (e.g., "USD", "NGN")
    pub bank_id: String, // Bank account ID from /api/bank_accounts
    pub reference: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct WithdrawResponse {
    transaction_id: String,
}

static SUPPORTED_CURRENCIES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(USD|NGN|GBP|EUR|CAD|AUD|JPY|CHF|CNY|SEK|NZD|MXN|SGD|HKD|NOK|KRW|TRY|INR|BRL|ZAR)$")
        .expect("Invalid currency regex")
});

#[utoipa::path(
    post,
    path = "/api/withdraw",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdrawal initiated", body = WithdrawResponse),
        (status = 400, description = "Invalid amount, insufficient balance, or bank not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Transaction"
)]
pub async fn withdraw(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, (StatusCode, String)> {
    info!(
        "Withdrawal request: user_id={}, amount={}, currency={}, bank_id={}",
        claims.sub, req.amount, req.currency, req.bank_id
    );

    // Validate input
    if req.amount <= 0.0 {
        error!("Invalid amount: {}", req.amount);
        return Err(ApiError::Payment("Amount must be positive".to_string()).into());
    }
    if !SUPPORTED_CURRENCIES.is_match(&req.currency) {
        error!("Invalid currency: {}", req.currency);
        return Err(ApiError::Payment("Invalid currency".to_string()).into());
    }
    let amount_cents = (req.amount * 100.0).round() as i64; // Amount in cents/kobo

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Parse bank ID
    let bnk_id = Uuid::parse_str(&req.bank_id).map_err(|e| {
        error!("Invalid bank ID: {}", e);
        ApiError::Payment("Invalid bank ID".to_string())
    })?;

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch sender wallet in the selected currency
    let sender_wallet = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(&req.currency))
        .first::<Wallet>(conn)
        .map_err(|e| {
            error!("Sender wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment(format!("Wallet not found for currency {}", req.currency))
            } else {
                ApiError::Database(e)
            }
        })?;

    // Validate balance
    if sender_wallet.balance < amount_cents {
        error!(
            "Insufficient balance: available={}, required={}",
            sender_wallet.balance, amount_cents
        );
        return Err(ApiError::Payment("Insufficient balance".to_string()).into());
    }

    // Fetch bank account
    let bank_account = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .filter(bank_accounts::id.eq(bnk_id))
        .filter(bank_accounts::is_verified.eq(true))
        .first::<BankAccount>(conn)
        .map_err(|e| {
            error!("Bank account lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Bank account not found or not verified".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    // Convert amount to NGN for Paystack
    let amount_ngn_kobo = if req.currency == "NGN" {
        amount_cents
    } else {
        let exchange_rate = get_exchange_rate(&req.currency, "NGN").await.map_err(|e| {
            // error!("Exchange rate fetch failed: {}", e);
            ApiError::Payment("Exchange rate fetch failed".to_string())
        })?;
        ((amount_cents as f64) * exchange_rate).round() as i64
    };

    // Check Paystack balance
    let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
        error!("PAYSTACK_SECRET_KEY not set");
        ApiError::Payment("Paystack key not set".to_string())
    })?;
    let client = Client::new();
    let balance_resp = client
        .get("https://api.paystack.co/balance")
        .header("Authorization", format!("Bearer {}", paystack_key))
        .send()
        .await
        .map_err(|e| {
            error!("Paystack API error: {}", e);
            ApiError::Payment(format!("Paystack API error: {}", e))
        })?;

    let balance_body = balance_resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            error!("Paystack response parsing error: {}", e);
            ApiError::Payment(format!("Paystack response error: {}", e))
        })?;
    info!("Paystack balance response: {:?}", balance_body);

    // Find NGN balance
    let ngn_balance = balance_body["data"]
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .find(|item| item["currency"].as_str() == Some("NGN"))
                .and_then(|item| item["balance"].as_i64())
        })
        .ok_or_else(|| {
            error!("No NGN balance found in Paystack response");
            ApiError::Payment("No NGN balance found".to_string())
        })?;
    info!("Paystack NGN balance: {} kobo", ngn_balance);

    // Estimate Paystack fee (simplified: ₦50 for transfers < ₦5,000, ₦100 for others)
    let paystack_fee = if amount_ngn_kobo < 500000 { 5000 } else { 10000 }; // In kobo
    let total_required = amount_ngn_kobo + paystack_fee;
    if ngn_balance < total_required {
        error!(
            "Insufficient Paystack balance: available={}, required={} (amount={} + fee={})",
            ngn_balance, total_required, amount_ngn_kobo, paystack_fee
        );
        return Err(ApiError::Payment(format!(
            "Insufficient Paystack balance: available ₦{:.2}, required ₦{:.2}",
            ngn_balance as f64 / 100.0,
            total_required as f64 / 100.0
        ))
            .into());
    }

    // Idempotency check: check if transaction with this reference already exists
    let existing_transaction = transactions::table
        .filter(transactions::reference.eq(req.reference))
        .first::<crate::models::models::Transaction>(conn)
        .optional()
        .map_err(|e| {
            error!("Database error checking idempotency: {}", e);
            ApiError::Database(e)
        })?;

    if let Some(tx) = existing_transaction {
        info!("Idempotent request: transaction {} already exists", tx.reference);
        return Ok(Json(WithdrawResponse {
            transaction_id: tx.reference.to_string(),
        }));
    }

    // Initiate Paystack transfer
    let reference = req.reference;
    let resp = client
        .post("https://api.paystack.co/transfer")
        .header("Authorization", format!("Bearer {}", paystack_key))
        .json(&serde_json::json!({
            "source": "balance",
            "reason": format!("Withdrawal from Payego in {}", req.currency),
            "amount": amount_ngn_kobo,
            "recipient": bank_account.paystack_recipient_code,
            "reference": reference.to_string(),
        }))
        .send()
        .await
        .map_err(|e| {
            error!("Paystack API error: {}", e);
            ApiError::Payment(format!("Paystack API error: {}", e))
        })?;

    let status = resp.status();
    let body = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            error!("Paystack response parsing error: {}", e);
            ApiError::Payment(format!("Paystack response error: {}", e))
        })?;
    info!("Paystack transfer response: {:?}", body);

    if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
        let message = body["message"]
            .as_str()
            .unwrap_or("Unknown Paystack error")
            .to_string();
        error!("Paystack withdrawal failed: {}", message);
        return Err(ApiError::Payment(format!("Paystack withdrawal failed: {}", message)).into());
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
        // Debit owner wallet
        diesel::update(wallets::table)
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(&req.currency))
            .set(wallets::balance.eq(wallets::balance - amount_cents))
            .execute(conn)
            .map_err(|e| {
                error!("Owner wallet update failed: {}", e);
                ApiError::Database(e)
            })?;
        info!(
            "Debited owner wallet: user_id={}, amount={}, currency={}",
            user_id, amount_cents, req.currency
        );

        // Insert transaction
        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id,
                recipient_id: None,
                amount: -amount_cents,
                transaction_type: "paystack_payout".to_string(),
                currency: req.currency.to_uppercase(),
                status: "pending".to_string(),
                provider: Some("paystack".to_string()),
                description: Some(format!("Withdrawal to bank {} in {}", bank_account.bank_code, req.currency)),
                reference,
            })
            .execute(conn)
            .map_err(|e| {
                error!("Withdrawal transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        Ok::<(), ApiError>(())
    })?;

    info!(
        "Withdrawal initiated: user_id={}, amount={}, currency={}, bank_id={}",
        user_id, req.amount, req.currency, bnk_id
    );
    Ok(Json(WithdrawResponse {
        transaction_id: reference.to_string(),
    }))
}

async fn get_exchange_rate(from_currency: &str, to_currency: &str) -> Result<f64, ApiError> {
    if from_currency == to_currency {
        return Ok(1.0);
    }
    let url = format!(
        "https://api.exchangerate-api.com/v4/latest/{}",
        from_currency
    );
    let client = Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| ApiError::Payment(e.to_string()))?;

    let body = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| ApiError::Payment(e.to_string()))?;

    let rate = body["rates"][to_currency]
        .as_f64()
        .ok_or(ApiError::Payment(
            "Invalid exchange rate response".to_string(),
        ))?;

    Ok(rate)
}