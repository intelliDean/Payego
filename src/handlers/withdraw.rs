use crate::AppState;
use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::models::user_models::{BankAccount, NewTransaction, Wallet};
use crate::schema::{bank_accounts, transactions, wallets};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use diesel::prelude::*;
use diesel::sql_types::Jsonb;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/withdraw",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdrawal initiated"),
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
) -> Result<StatusCode, (StatusCode, String)> {
    info!(
        "Withdrawal request: user_id={}, amount={}, currency={}",
        claims.sub, req.amount, req.currency
    );

    // Validate amount
    if req.amount <= 0.0 {
        error!("Invalid amount: {}", req.amount);
        return Err(ApiError::Payment("Amount must be positive".to_string()).into());
    }
    let amount_ngn_cents = (req.amount * 100.0).round() as i64; // Amount in NGN cents

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Parse user ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Fetch sender wallet in the selected currency
    let sender_wallet = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(&req.currency))
        .first::<Wallet>(&mut conn)
        .map_err(|e| {
            error!("Sender wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment(format!("Wallet not found for currency {}", req.currency))
            } else {
                ApiError::Database(e)
            }
        })?;

    // Fetch current exchange rate
    let exchange_rate = get_exchange_rate(&req.currency, "NGN").await.map_err(|e| {
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
        return Err(ApiError::Payment("Insufficient balance".to_string()).into());
    }

    // Fetch bank account
    let bank_account = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .first::<BankAccount>(&mut conn)
        .map_err(|e| {
            error!("Bank account lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Bank account not found".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    // Initiate Paystack transfer
    let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
        error!("PAYSTACK_SECRET_KEY not set");
        ApiError::Token("Paystack key not set".to_string())
    })?;
    let client = Client::new();
    let reference = Uuid::new_v4();
    let resp = client
        .post("https://api.paystack.co/transfer")
        .header("Authorization", format!("Bearer {}", paystack_key))
        .json(&serde_json::json!({
            "source": "balance",
            "reason": format!("Withdrawal from Payego in {}", req.currency),
            "amount": amount_ngn_cents,
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
    let body = resp.json::<serde_json::Value>().await.map_err(|e| {
        error!("Paystack response parsing error: {}", e);
        ApiError::Payment(format!("Paystack response error: {}", e))
    })?;

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
            .set(wallets::balance.eq(wallets::balance - amount_to_deduct))
            .execute(conn)
            .map_err(|e| {
                error!("Owner wallet update failed: {}", e);
                ApiError::Database(e)
            })?;
        info!(
            "Debited owner wallet: user_id = {}, amount = {}, currency = {}",
            user_id, amount_to_deduct, req.currency
        );

        // Insert transaction
        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id,
                recipient_id: None,
                amount: -amount_to_deduct,
                transaction_type: "paystack_payout".to_string(),
                status: "pending".to_string(),
                provider: Some("paystack".to_string()),
                description: Some(format!("Withdrawal in {} to bank", req.currency)),
                reference,
                // metadata: Some(Jsonb(
                //     json!({ "transfer_code": transfer_code, "exchange_rate": exchange_rate }),
                // )),
            })
            .execute(conn)
            .map_err(|e| {
                error!("Withdrawal transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        Ok::<(), ApiError>(())
    })?;
    // .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!(
        "Withdrawal initiated: user_id={}, amount_ngn={}, currency={}, exchange_rate={}",
        user_id, req.amount, req.currency, exchange_rate
    );
    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
pub struct WithdrawRequest {
    amount: f64,      // Amount in NGN
    currency: String, // Currency to deduct from (e.g., "USD")
}

async fn get_exchange_rate(from_currency: &str, to_currency: &str) -> Result<f64, ApiError> {
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
