use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, BankAccount, BankRequest, NewBankAccount};
use payego_primitives::schema::bank_accounts;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

pub struct BankService;

impl BankService {
    pub async fn add_bank_account(
        state: &AppState,
        user_id_val: Uuid,
        req: BankRequest,
    ) -> Result<BankAccount, ApiError> {
        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let account_details =
            Self::resolve_account_details(state, &req.bank_code, &req.account_number).await?;
        let account_name = account_details["account_name"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("Missing account name".to_string()))?
            .to_string();

        let paystack_key = state.paystack_secret_key.expose_secret();
        let client = Client::new();

        let recipient_resp = client
            .post(format!("{}/transferrecipient", state.paystack_api_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&json!({
                "type": "nuban",
                "name": account_name,
                "account_number": req.account_number,
                "bank_code": req.bank_code,
                "currency": "NGN"
            }))
            .send()
            .await
            .map_err(|e: reqwest::Error| {
                ApiError::Payment(format!("Recipient creation failed: {}", e))
            })?;

        let r_body = recipient_resp
            .json::<Value>()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Parsing failed: {}", e)))?;
        let recipient_code = r_body["data"]["recipient_code"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("Missing recipient code".to_string()))?
            .to_string();

        let new_account = NewBankAccount {
            id: Uuid::new_v4(),
            user_id: user_id_val,
            bank_name: Some(req.bank_name),
            account_number: req.account_number,
            account_name: Some(account_name),
            bank_code: req.bank_code,
            paystack_recipient_code: Some(recipient_code),
            is_verified: true,
        };

        let account = diesel::insert_into(bank_accounts::table)
            .values(&new_account)
            .get_result::<BankAccount>(&mut conn)
            .map_err(ApiError::from)?;

        info!("Bank account added successfully for user: {}", user_id_val);
        Ok(account)
    }

    pub async fn resolve_account_details(
        state: &AppState,
        bank_code: &str,
        account_number: &str,
    ) -> Result<Value, ApiError> {
        let paystack_key = state.paystack_secret_key.expose_secret();
        let client = Client::new();

        let resp = client
            .get(format!(
                "{}/bank/resolve?account_number={}&bank_code={}",
                state.paystack_api_url, account_number, bank_code
            ))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .send()
            .await
            .map_err(|e: reqwest::Error| {
                ApiError::Payment(format!("Paystack resolve failed: {}", e))
            })?;

        let status = resp.status();
        let body = resp
            .json::<Value>()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Parsing failed: {}", e)))?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
            return Err(ApiError::Payment(
                "Invalid bank account details".to_string(),
            ));
        }

        Ok(body["data"].clone())
    }

    pub async fn get_bank_accounts(
        state: &AppState,
        user_id_val: Uuid,
    ) -> Result<Vec<BankAccount>, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e: r2d2::Error| ApiError::DatabaseConnection(e.to_string()))?;

        let accounts = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id_val))
            .load::<BankAccount>(&mut conn)
            .map_err(ApiError::from)?;

        Ok(accounts)
    }
}
