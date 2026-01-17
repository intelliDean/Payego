use std::str::FromStr;
use std::sync::Arc;
use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::schema::{bank_accounts, banks};
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};
use tracing::log::warn;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::bank::{Bank, BankAccount, NewBank, NewBankAccount};
use payego_primitives::models::bank_dtos::BankRequest;
use payego_primitives::models::dtos::dtos::{ PaystackRecipientResponse, ResolvedAccount};
use payego_primitives::models::enum_types::CurrencyCode;

static ACCOUNT_CACHE: Lazy<DashMap<String, (ResolvedAccount, Instant)>> = Lazy::new(DashMap::new);
const TTL: Duration = Duration::from_secs(60 * 10); // 10 minutes

#[derive(Debug, Deserialize)]
struct PaystackResolveResponse {
    status: bool,
    message: String,
    data: Option<PaystackAccountData>,
}

#[derive(Debug, Deserialize)]
struct PaystackAccountData {
    account_name: String,
}


#[derive(Debug, Deserialize)]
pub struct PaystackResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct PaystackBank {
    pub id: i64,
    pub name: String,
    pub code: String,

    // #[serde(default)]
    pub currency: Option<String>,

    // #[serde(default)]
    pub country: Option<String>,

    #[serde(rename = "active", default)]
    pub is_active: bool,
}


pub struct BankService;

impl BankService {
    pub async fn create_bank_account(
        state: &AppState,
        user_id_val: Uuid,
        req: BankRequest,
    ) -> Result<BankAccount, ApiError> {
        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // Idempotency check
        if let Ok(existing) = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id_val))
            .filter(bank_accounts::bank_code.eq(&req.bank_code))
            .filter(bank_accounts::account_number.eq(&req.account_number))
            .first::<BankAccount>(&mut conn)
        {
            return Ok(existing);
        }

        let account_details =
            Self::resolve_account_details(state, &req.bank_code, &req.account_number).await?;

        let account_name = account_details.account_name;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|_| ApiError::Internal("HTTP client error".into()))?;

        let resp = client
            .post(format!("{}/transferrecipient", state.config.paystack_details.paystack_api_url))
            .bearer_auth(state.config.paystack_details.paystack_secret_key.expose_secret())
            .json(&json!({
                "type": "nuban",
                "name": account_name,
                "account_number": req.account_number,
                "bank_code": req.bank_code,
                "currency": "NGN"
            }))
            .send()
            .await
            .map_err(|_| ApiError::Payment("Paystack recipient creation failed".into()))?;

        let body: PaystackRecipientResponse = resp
            .json()
            .await
            .map_err(|_| ApiError::Payment("Invalid Paystack response".into()))?;

        let new_account = NewBankAccount {
            user_id: user_id_val,
            bank_name: Some(&req.bank_name),
            account_number: &req.account_number,
            account_name: Some(&*account_name),
            bank_code: &*req.bank_code,
            provider_recipient_id: Some(&*body.data.recipient_code),
            is_verified: true, // rename later
        };

        let account = diesel::insert_into(bank_accounts::table)
            .values(&new_account)
            .get_result(&mut conn)?;

        Ok(account)
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



    pub async fn initialize_banks(
        state: &Arc<AppState>,
        conn: &mut PgConnection,
    ) -> Result<bool, ApiError> {
        // 1. Idempotency check
        let existing: i64 = banks::table.count().get_result(conn)?;
        if existing > 0 {
            info!("Banks already initialized ({} records exist)", existing);
            return Ok(false);
        }

        info!("Starting Paystack bank initialization...");

        let secret_key = state.config.paystack_details.paystack_secret_key.expose_secret();

        // Safety check – prevent sending empty bearer token
        if secret_key.trim().is_empty() {
            return Err(ApiError::Internal("Paystack secret key is empty".into()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| ApiError::Payment(format!("Failed to create HTTP client: {}", e)))?;

        let url = "https://api.paystack.co/bank?country=nigeria";

        let resp = client
            .get(url)
            .bearer_auth(secret_key)
            .header("User-Agent", "Payego/1.0 (Rust backend)")
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("Failed to reach Paystack: {}", e)))?;

        let status = resp.status();
        let body_text = resp.text().await
            .unwrap_or_else(|_| "<empty response>".to_string());

        if !status.is_success() {
            error!(
            http_status = status.as_u16(),
            response_body = %body_text,
            "Paystack bank list request failed"
        );

            return Err(ApiError::Payment(format!(
                "Paystack API error {}: {}",
                status, body_text.chars().take(200).collect::<String>()
            )));
        }

        // Try to parse - if it fails we get useful error
        let body: PaystackResponse<Vec<PaystackBank>> = serde_json::from_str(&body_text)
            .map_err(|e| {
                error!(
                parse_error = %e,
                response_body_first_500 = %body_text.chars().take(500).collect::<String>(),
                "Failed to parse Paystack bank response as JSON"
            );
                ApiError::Payment(format!("Invalid JSON from Paystack: {}", e))
            })?;

        //this is returning the banks from paystack
        // info!("Body: {:?}", body);

        if !body.status {
            return Err(ApiError::Payment(format!("Paystack error: {}", body.message)));
        }

        if body.data.is_empty() {
            warn!("Paystack returned empty bank list - possible API issue");
            return Err(ApiError::Payment("Paystack returned no banks".into()));
        }

        let banks_to_insert: Vec<NewBank> = body.data.into_iter().map(|b| {

            NewBank {
                id: b.id,
                name: b.name,
                code: b.code,
                currency: CurrencyCode::parse(&b.currency.unwrap()).unwrap(),
                country: b.country.unwrap(),
                is_active: b.is_active,
            }
        }).collect();

        info!("OneBank: {:?}", banks_to_insert[0].currency);

        let inserted = diesel::insert_into(banks::table)
            .values(&banks_to_insert)
            .on_conflict(banks::code)
            .do_nothing()
            .execute(conn)?;

        info!("Successfully initialized {} Nigerian banks from Paystack", inserted);

        Ok(true)
    }
    pub async fn resolve_account_details(
        state: &AppState,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ResolvedAccount, ApiError> {
        let cache_key = format!("{bank_code}:{account_number}");

        if let Some(cached) = get(&cache_key) {
            info!("Bank resolve cache hit: {}", cache_key);
            return Ok(cached);
        }

        let resp = state.http_client
            .get(format!("{}/bank/resolve", state.config.paystack_details.paystack_api_url))
            .query(&[("account_number", account_number), ("bank_code", bank_code)])
            .bearer_auth(&state.config.paystack_details.paystack_secret_key.expose_secret())
            .send()
            .await?;

        let body: PaystackResolveResponse = resp.json().await?;

        if !body.status {
            warn!("Paystack resolve failed: {}", body.message);
            return Err(ApiError::Payment(body.message));
        }

        let data = body
            .data
            .ok_or_else(|| ApiError::Payment("Missing account data from Paystack".into()))?;

        let resolved = ResolvedAccount {
            account_name: data.account_name,
            bank_code: bank_code.to_string(),
            account_number: account_number.to_string(),
        };

        set(cache_key, resolved.clone());

        Ok(resolved)
    }
}

// payego_core::cache::bank_cache.rs
pub fn get(key: &str) -> Option<ResolvedAccount> {
    ACCOUNT_CACHE.get(key).and_then(|entry| {
        if entry.value().1.elapsed() < TTL {
            Some(entry.value().0.clone())
        } else {
            ACCOUNT_CACHE.remove(key);
            None
        }
    })
}

pub fn set(key: String, value: ResolvedAccount) {
    ACCOUNT_CACHE.insert(key, (value, Instant::now()));
}


