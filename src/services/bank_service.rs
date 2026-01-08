use crate::error::ApiError;
use crate::models::models::NewBankAccount;
use crate::schema::{bank_accounts, wallets};
use diesel::prelude::*;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, debug, info};
use uuid::Uuid;
use crate::AppState;
use crate::handlers::bank::{BankRequest, BankResponse};

pub struct BankService;

impl BankService {
    pub async fn add_bank_account(
        state: Arc<AppState>,
        user_id: Uuid,
        req: BankRequest,
    ) -> Result<BankResponse, ApiError> {
        info!(
            "Add bank request: user_id = {}, bank_code = {}, account_number = {}",
            user_id, req.bank_code, req.account_number
        );

        let mut conn = state.db.get().map_err(|e| {
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
            .execute(&mut conn)
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
            .first::<i64>(&mut conn)
            .map(|count| count > 0)
            .map_err(|e| {
                error!("Bank account lookup failed: {}", e);
                ApiError::Database(e)
            })?;

        if existing {
            return Err(ApiError::Payment("Bank account already exists".to_string()));
        }

        // Verify bank account with Paystack
        let account_name = Self::resolve_account_details(&req.bank_code, &req.account_number).await?;
        debug!("Resolved account_name: {}", account_name);

        // Create Paystack transfer recipient
        let recipient_code = Self::create_transfer_recipient(&req.bank_code, &req.account_number, &account_name).await?;

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
                paystack_recipient_code: Some(recipient_code),
                is_verified: true,
            })
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to add bank account: {}", e);
                ApiError::Database(e)
            })?;

        Ok(BankResponse {
            transaction_id: bank_account_id.to_string(),
            account_name,
        })
    }

    pub async fn resolve_account_details(bank_code: &str, account_number: &str) -> Result<String, ApiError> {
        let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
            error!("PAYSTACK_SECRET_KEY not set");
            ApiError::Payment("Paystack key not set".to_string())
        })?;
        let client = Client::new();
        let resolve_resp = client
            .get(format!(
                "https://api.paystack.co/bank/resolve?account_number={}&bank_code={}",
                account_number, bank_code
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
            .json::<Value>()
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
            return Err(ApiError::Payment(format!("Paystack account resolution failed: {}", message)));
        }

        let account_name = resolve_body["data"]["account_name"]
            .as_str()
            .ok_or_else(|| {
                error!("Missing account_name in Paystack resolve response");
                ApiError::Payment("Invalid Paystack resolve response: missing account_name".to_string())
            })?
            .to_string();
        
        Ok(account_name)
    }

    async fn create_transfer_recipient(bank_code: &str, account_number: &str, account_name: &str) -> Result<String, ApiError> {
        let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
            ApiError::Payment("Paystack key not set".to_string())
        })?;
        let client = Client::new();
        let recipient_resp = client
            .post("https://api.paystack.co/transferrecipient")
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&serde_json::json!({
                "type": "nuban",
                "name": account_name,
                "account_number": account_number,
                "bank_code": bank_code,
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
            .json::<Value>()
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
            return Err(ApiError::Payment(format!("Paystack transferrecipient creation failed: {}", message)));
        }

        let recipient_code = recipient_body["data"]["recipient_code"]
            .as_str()
            .ok_or_else(|| {
                error!("Missing recipient_code in Paystack transferrecipient response");
                ApiError::Payment("Invalid Paystack response: missing recipient_code".to_string())
            })?
            .to_string();
            
        Ok(recipient_code)
    }
}
