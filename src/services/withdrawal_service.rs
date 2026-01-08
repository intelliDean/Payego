use crate::error::ApiError;
use crate::models::models::{AppState, BankAccount, NewTransaction, Transaction, Wallet};
use crate::schema::{bank_accounts, transactions, wallets};
use diesel::prelude::*;
use reqwest::Client;
use serde_json::{Value, json};
use std::sync::{Arc, LazyLock};
use regex::Regex;
use tracing::{error, debug, info};
use uuid::Uuid;
use crate::handlers::withdraw::{WithdrawRequest, WithdrawResponse};

static SUPPORTED_CURRENCIES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(USD|NGN|GBP|EUR|CAD|AUD|JPY|CHF|CNY|SEK|NZD|MXN|SGD|HKD|NOK|KRW|TRY|INR|BRL|ZAR)$")
        .expect("Invalid currency regex")
});

pub struct WithdrawalService;

impl WithdrawalService {
    pub async fn initiate_withdrawal(
        state: Arc<AppState>,
        user_id: Uuid,
        req: WithdrawRequest,
    ) -> Result<WithdrawResponse, ApiError> {
        info!(
            "Withdrawal request: user_id={}, amount={}, currency={}, bank_id={}",
            user_id, req.amount, req.currency, req.bank_id
        );

        // Validate input
        if req.amount <= 0.0 {
            error!("Invalid amount: {}", req.amount);
            return Err(ApiError::Payment("Amount must be positive".to_string()));
        }
        if !SUPPORTED_CURRENCIES.is_match(&req.currency) {
            error!("Invalid currency: {}", req.currency);
            return Err(ApiError::Payment("Invalid currency".to_string()));
        }
        let amount_cents = (req.amount * 100.0).round() as i64;

        // Parse bank ID
        let bnk_id = Uuid::parse_str(&req.bank_id).map_err(|e| {
            error!("Invalid bank ID: {}", e);
            ApiError::Payment("Invalid bank ID".to_string())
        })?;

        // Get database connection
        let mut conn = state.db.get().map_err(|e| {
            error!("Database connection failed: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // Idempotency check with metadata
        let existing_transaction = transactions::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ").bind::<diesel::sql_types::Text, _>(&req.idempotency_key))
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(ApiError::Database)?;

        if let Some(tx) = existing_transaction {
            info!("Idempotent request: transaction {} already exists for key {}", tx.reference, req.idempotency_key);
            return Ok(WithdrawResponse {
                transaction_id: tx.reference.to_string(),
            });
        }

        // Fetch sender wallet
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

        // Validate balance
        if sender_wallet.balance < amount_cents {
            error!(
                "Insufficient balance: available={}, required={}",
                sender_wallet.balance, amount_cents
            );
            return Err(ApiError::Payment("Insufficient balance".to_string()));
        }

        // Fetch bank account
        let bank_account = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id))
            .filter(bank_accounts::id.eq(bnk_id))
            .filter(bank_accounts::is_verified.eq(true))
            .first::<BankAccount>(&mut conn)
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
            let exchange_rate = Self::get_exchange_rate(&state.exchange_api_url, &req.currency, "NGN").await?;
            ((amount_cents as f64) * exchange_rate).round() as i64
        };

        // Paystack interaction
        Self::handle_paystack_transfer(
            &state, 
            amount_ngn_kobo, 
            &bank_account, 
            req.reference, 
            &req.currency
        ).await?;

        // Atomic DB transaction
        let mut conn = state.db.get().map_err(|e| {
             ApiError::DatabaseConnection(e.to_string())
        })?;

        conn.transaction(|conn| {
            // Debit owner wallet
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.currency))
                .set(wallets::balance.eq(wallets::balance - amount_cents))
                .execute(conn)
                .map_err(ApiError::Database)?;

            // Insert transaction
            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: -amount_cents,
                    transaction_type: "paystack_payout".to_string(),
                    currency: req.currency.to_uppercase(),
                    status: "pending".to_string(), // In reality, we might want to check Paystack status
                    provider: Some("paystack".to_string()),
                    description: Some(format!("Withdrawal to bank {} in {}", bank_account.bank_code, req.currency)),
                    reference: req.reference,
                    metadata: Some(json!({
                        "idempotency_key": req.idempotency_key
                    })),
                })
                .execute(conn)
                .map_err(ApiError::Database)?;

            Ok::<(), ApiError>(())
        })?;

        info!(
            "Withdrawal initiated: user_id={}, amount={}, currency={}, bank_id={}",
            user_id, req.amount, req.currency, bnk_id
        );

        Ok(WithdrawResponse {
            transaction_id: req.reference.to_string(),
        })
    }

    async fn handle_paystack_transfer(
        state: &AppState,
        amount_ngn_kobo: i64,
        bank_account: &BankAccount,
        reference: Uuid,
        currency: &str
    ) -> Result<(), ApiError> {
        let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
            error!("PAYSTACK_SECRET_KEY not set");
            ApiError::Payment("Paystack key not set".to_string())
        })?;

        let client = Client::new();

        let base_url = &state.paystack_api_url;

        // Check Paystack balance
        let balance_resp = client
            .get(format!("{}/balance", base_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("Paystack API error: {}", e)))?;

        let balance_body = balance_resp
            .json::<Value>()
            .await
            .map_err(|e| ApiError::Payment(format!("Paystack response error: {}", e)))?;

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

        // Fee logic
        let paystack_fee = if amount_ngn_kobo < 500000 { 5000 } else { 10000 };
        let total_required = amount_ngn_kobo + paystack_fee;

        if ngn_balance < total_required {
             return Err(ApiError::Payment(format!(
                "Insufficient Paystack balance: available ₦{:.2}, required ₦{:.2}",
                ngn_balance as f64 / 100.0,
                total_required as f64 / 100.0
            )));
        }

        // Initiate Transfer
        let resp = client
            .post(format!("{}/transfer", base_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&serde_json::json!({
                "source": "balance",
                "reason": format!("Withdrawal from Payego in {}", currency),
                "amount": amount_ngn_kobo,
                "recipient": bank_account.paystack_recipient_code,
                "reference": reference.to_string(),
            }))
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("Paystack API error: {}", e)))?;

        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e| ApiError::Payment(format!("Paystack response error: {}", e)))?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
             let message = body["message"]
                .as_str()
                .unwrap_or("Unknown Paystack error")
                .to_string();
            return Err(ApiError::Payment(format!("Paystack withdrawal failed: {}", message)));
        }

        let transfer_code = body["data"]["transfer_code"]
            .as_str()
            .ok_or_else(|| {
                 ApiError::Payment("Invalid Paystack response: missing transfer_code".to_string())
            })?;
        
        debug!("Paystack transfer_code: {}", transfer_code);
        Ok(())
    }

    async fn get_exchange_rate(base_url: &str, from_currency: &str, to_currency: &str) -> Result<f64, ApiError> {
        if from_currency == to_currency {
            return Ok(1.0);
        }
        let url = format!("{}/{}", base_url, from_currency);
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
}
