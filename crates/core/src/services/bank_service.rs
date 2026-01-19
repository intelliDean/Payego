use dashmap::DashMap;
use diesel::prelude::*;
use once_cell::sync::Lazy;
pub use payego_primitives::{
    error::ApiError,
    models::{
        app_state::app_state::AppState,
        bank::{Bank, NewBank, NewBankAccount},
        bank_dtos::{
            BankDto, BankListResponse, PaystackBank, PaystackResolveResponse, PaystackResponse, ResolveAccountRequest, ResolveAccountResponse
        },
        dtos::bank_dtos::ResolvedAccount,
        enum_types::CurrencyCode,
    },
    schema::banks,
};
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::log::warn;
use tracing::{error, info};

static ACCOUNT_CACHE: Lazy<DashMap<String, (ResolvedAccount, Instant)>> = Lazy::new(DashMap::new);
const TTL: Duration = Duration::from_secs(60 * 10); // 10 minutes

pub struct BankService;

impl BankService {
    pub async fn initialize_banks(state: &Arc<AppState>) -> Result<bool, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        // 1. Idempotency check
        let existing: i64 = banks::table.count().get_result(&mut conn)?;
        if existing > 0 {
            info!("Banks already initialized ({} records exist)", existing);
            return Ok(false);
        }

        info!("Starting Paystack bank initialization...");

        let secret_key = state
            .config
            .paystack_details
            .paystack_secret_key
            .expose_secret();

        // Safety check – prevent sending empty bearer token
        if secret_key.trim().is_empty() {
            return Err(ApiError::Internal("Paystack secret key is empty".into()));
        }

        let url = "https://api.paystack.co/bank?country=nigeria";

        let resp = state
            .http_client
            .get(url)
            .bearer_auth(secret_key)
            .header("User-Agent", "Payego/1.0 (Rust backend)")
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("Failed to reach Paystack: {}", e)))?;

        let status = resp.status();
        let body_text = resp
            .text()
            .await
            .unwrap_or_else(|_| "<empty response>".to_string());

        if !status.is_success() {
            error!(
                http_status = status.as_u16(),
                response_body = %body_text,
                "Paystack bank list request failed"
            );

            return Err(ApiError::Payment(format!(
                "Paystack API error {}: {}",
                status,
                body_text.chars().take(200).collect::<String>()
            )));
        }

        // Try to parse - if it fails we get useful error
        let body: PaystackResponse<Vec<PaystackBank>> =
            serde_json::from_str(&body_text).map_err(|e| {
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
            return Err(ApiError::Payment(format!(
                "Paystack error: {}",
                body.message
            )));
        }

        if body.data.is_empty() {
            warn!("Paystack returned empty bank list - possible API issue");
            return Err(ApiError::Payment("Paystack returned no banks".into()));
        }

        let banks_to_insert = Self::insert_banks(body);

        let inserted = diesel::insert_into(banks::table)
            .values(&banks_to_insert)
            .on_conflict(banks::code)
            .do_nothing()
            .execute(&mut conn)?;

        info!(
            "Successfully initialized {} Nigerian banks from Paystack",
            inserted
        );

        Ok(true)
    }

    fn insert_banks(body: PaystackResponse<Vec<PaystackBank>>) -> Vec<NewBank> {
        let banks_to_insert: Vec<NewBank> = body
            .data
            .into_iter()
            .filter_map(|b| {
                let currency = match b.currency.as_deref() {
                    Some(raw) => match CurrencyCode::parse(raw) {
                        Ok(c) => c,
                        Err(_) => {
                            warn!(
                            "Skipping bank due to invalid currency (id={bank_id}, code={bank_code})",
                            bank_id = b.id,
                            bank_code = b.code,
                        );
                            return None;
                        }
                    },
                    None => {
                        warn!(
                            "Skipping bank due to missing country (id={bank_id}, code={bank_code})",
                            bank_id = b.id,
                            bank_code = b.code,
                        );
                        return None;
                    }
                };

                let country = match b.country {
                    Some(c) => c,
                    None => {
                        warn!(
                            "Skipping bank due to missing country (id={bank_id}, code={bank_code})",
                            bank_id = b.id,
                            bank_code = b.code,
                        );
                        return None;
                    }
                };

                Some(NewBank {
                    id: b.id,
                    name: b.name,
                    code: b.code,
                    currency,
                    country,
                    is_active: b.is_active,
                })
            })
            .collect();
        banks_to_insert
    }

    pub async fn resolve_account_details(
        state: &AppState,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ResolvedAccount, ApiError> {
        let cache_key = format!("{bank_code}:{account_number}");

        if let Some(cached) = Self::get(&cache_key) {
            info!("Bank resolve cache hit: {}", cache_key);
            return Ok(cached);
        }

        let resp = state
            .http_client
            .get(format!(
                "{}/bank/resolve",
                state.config.paystack_details.paystack_api_url
            ))
            .query(&[("account_number", account_number), ("bank_code", bank_code)])
            .bearer_auth(
                &state
                    .config
                    .paystack_details
                    .paystack_secret_key
                    .expose_secret(),
            )
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

        Self::set(cache_key, resolved.clone());

        Ok(resolved)
    }

    pub async fn list_banks(state: &AppState) -> Result<BankListResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("banks.list: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let banks = banks::table
            .filter(banks::country.eq(&state.config.default_country))
            .filter(banks::is_active.eq(true))
            .order(banks::name.asc())
            .load::<Bank>(&mut conn)
            .map_err(|_| {
                error!("banks.list: query failed");
                ApiError::Internal("Failed to fetch banks".into())
            })?;

        Ok(BankListResponse {
            banks: banks.into_iter().map(|b| BankDto::from(b)).collect(),
        })
    }

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
}
