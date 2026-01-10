use diesel::prelude::*;
use diesel::dsl::sql;
use diesel::sql_types::BigInt;
use payego_primitives::schema::{wallets, transactions, bank_accounts};
use payego_primitives::models::{AppState, Wallet, BankAccount, NewTransaction, WithdrawRequest, WithdrawResponse};
use payego_primitives::error::ApiError;
use uuid::Uuid;
use tracing::{info, error, debug};
use serde_json::{json, Value};
use reqwest::Client;

pub struct WithdrawalService;

impl WithdrawalService {
    pub async fn withdraw(
        state: &AppState,
        user_id: Uuid,
        bnk_id: Uuid,
        req: WithdrawRequest,
    ) -> Result<WithdrawResponse, ApiError> {
        let amount_cents = (req.amount * 100.0) as i64;

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let sender_wallet = wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(&req.currency))
            .first::<Wallet>(&mut conn)
            .map_err(|e: diesel::result::Error| {
                error!("Sender wallet lookup failed: {}", e);
                if e.to_string().contains("not found") {
                    ApiError::Payment(format!("Wallet not found for currency {}", req.currency))
                } else {
                    ApiError::from(e)
                }
            })?;

        if sender_wallet.balance < amount_cents {
            error!(
                "Insufficient balance: available={}, required={}",
                sender_wallet.balance, amount_cents
            );
            return Err(ApiError::Payment("Insufficient balance".to_string()));
        }

        let bank_account = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id))
            .filter(bank_accounts::id.eq(bnk_id))
            .filter(bank_accounts::is_verified.eq(true))
            .first::<BankAccount>(&mut conn)
            .map_err(|e: diesel::result::Error| {
                error!("Bank account lookup failed: {}", e);
                if e.to_string().contains("not found") {
                    ApiError::Payment("Bank account not found or not verified".to_string())
                } else {
                    ApiError::from(e)
                }
            })?;

        let amount_ngn_kobo = if req.currency == "NGN" {
            amount_cents
        } else {
            let exchange_rate =
                Self::get_exchange_rate(&state.exchange_api_url, &req.currency, "NGN").await?;
            ((amount_cents as f64) * exchange_rate).round() as i64
        };

        Self::handle_paystack_transfer(
            &state,
            amount_ngn_kobo,
            &bank_account,
            req.reference,
            &req.currency,
        )
        .await?;

        conn.transaction::<(), ApiError, _>(|conn| {
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.currency))
                .set(wallets::balance.eq(sql::<BigInt>("balance - ").bind::<BigInt, _>(amount_cents)))
                .execute(conn)
                .map_err(|e: diesel::result::Error| ApiError::from(e))?;

            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: -amount_cents,
                    transaction_type: "paystack_payout".to_string(),
                    currency: req.currency.to_uppercase(),
                    status: "pending".to_string(),
                    provider: Some("paystack".to_string()),
                    description: Some(format!(
                        "Withdrawal to bank {} in {}",
                        bank_account.bank_code, req.currency
                    )),
                    reference: req.reference,
                    metadata: Some(json!({
                        "idempotency_key": req.idempotency_key
                    })),
                })
                .execute(conn)
                .map_err(|e: diesel::result::Error| ApiError::from(e))?;

            Ok(())
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
        currency: &str,
    ) -> Result<(), ApiError> {
        let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|e: std::env::VarError| {
            error!("PAYSTACK_SECRET_KEY not set: {}", e);
            ApiError::Payment("Paystack key not set".to_string())
        })?;

        let client = Client::new();
        let base_url = &state.paystack_api_url;

        let balance_resp = client
            .get(format!("{}/balance", base_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Paystack API error: {}", e)))?;

        let balance_body = balance_resp
            .json::<Value>()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Paystack response error: {}", e)))?;

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

        let paystack_fee = if amount_ngn_kobo < 500000 {
            5000
        } else {
            10000
        };
        let total_required = amount_ngn_kobo + paystack_fee;

        if ngn_balance < total_required {
            return Err(ApiError::Payment(format!(
                "Insufficient Paystack balance: available ₦{:.2}, required ₦{:.2}",
                ngn_balance as f64 / 100.0,
                total_required as f64 / 100.0
            )));
        }

        let resp = client
            .post(format!("{}/transfer", base_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&json!({
                "source": "balance",
                "reason": format!("Withdrawal from Payego in {}", currency),
                "amount": amount_ngn_kobo,
                "recipient": bank_account.paystack_recipient_code,
                "reference": reference.to_string(),
            }))
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Paystack API error: {}", e)))?;

        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Paystack response error: {}", e)))?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
            let message = body["message"]
                .as_str()
                .unwrap_or("Unknown Paystack error")
                .to_string();
            return Err(ApiError::Payment(format!(
                "Paystack withdrawal failed: {}",
                message
            )));
        }

        let transfer_code = body["data"]["transfer_code"].as_str().ok_or_else(|| {
            ApiError::Payment("Invalid Paystack response: missing transfer_code".to_string())
        })?;

        debug!("Paystack transfer_code: {}", transfer_code);
        Ok(())
    }

    async fn get_exchange_rate(
        base_url: &str,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<f64, ApiError> {
        if from_currency == to_currency {
            return Ok(1.0);
        }
        let url = format!("{}/{}", base_url, from_currency);
        let client = Client::new();
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(e.to_string()))?;

        let body = resp
            .json::<Value>()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(e.to_string()))?;

        let rate = body["rates"][to_currency]
            .as_f64()
            .ok_or(ApiError::Payment(
                "Invalid exchange rate response".to_string(),
            ))?;

        Ok(rate)
    }
}
