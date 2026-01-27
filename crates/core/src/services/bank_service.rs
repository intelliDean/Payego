
pub use payego_primitives::{
    error::ApiError,
    models::{
        app_state::AppState,
        bank::{Bank, NewBank, NewBankAccount},
        bank_dtos::{
            BankDto, BankListResponse, PaystackBank, PaystackResolveResponse, PaystackResponse,
        },
        dtos::bank_dtos::ResolvedAccount,
        enum_types::CurrencyCode,
    },
    schema::banks,
};
use crate::repositories::bank_repository::BankRepository;
use reqwest::Url;
use secrecy::ExposeSecret;
use std::sync::Arc;
use tracing::log::warn;
use tracing::{error, info};

pub struct BankService;

impl BankService {
    pub async fn initialize_banks(state: &Arc<AppState>) -> Result<bool, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        // idempotency check
        let existing = BankRepository::count(&mut conn)?;
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

        // safety check to prevent sending empty bearer token
        if secret_key.trim().is_empty() {
            return Err(ApiError::Internal("Paystack secret key is empty".into()));
        }

        //to parse the url
        let mut url = Url::parse(&state.config.paystack_details.paystack_api_url)
            .map_err(|_| ApiError::Internal("Invalid Paystack base URL".into()))?;

        url.set_path("bank");
        url.query_pairs_mut()
            .append_pair("country", &state.config.default_country.to_lowercase());

        let resp = state
            .http_client
            .get(url)
            .bearer_auth(
                state
                    .config
                    .paystack_details
                    .paystack_secret_key
                    .expose_secret(),
            )
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

        let inserted = BankRepository::create_many(&mut conn, banks_to_insert)?;

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

    pub async fn list_banks(state: &AppState) -> Result<BankListResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("banks.list: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let banks = BankRepository::list_active_by_country(&mut conn, &state.config.default_country)?;

        Ok(BankListResponse {
            banks: banks.into_iter().map(BankDto::from).collect(),
        })
    }
}
