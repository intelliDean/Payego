use crate::client::PaystackClient;
use dashmap::DashMap;

use crate::repositories::bank_account_repository::BankAccountRepository;
use once_cell::sync::Lazy;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{app_state::AppState, bank::BankAccount},
    models::{
        bank::NewBankAccount,
        dtos::bank_dtos::{
            BankAccountResponse, BankAccountsResponse, BankRequest, DeleteResponse,
            PaystackRecipientResponse, PaystackResolveResponse, ResolveAccountRequest,
            ResolveAccountResponse, ResolvedAccount,
        },
        enum_types::{CurrencyCode, PaymentState},
    },
    schema::bank_accounts,
};
use regex::Regex;
use reqwest::Url;
use secrecy::ExposeSecret;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

static ACCOUNT_NUMBER_RE: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| Regex::new(r"^\d{10}$"));

static BANK_CODE_RE: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| Regex::new(r"^\d{3,10}$"));
static ACCOUNT_CACHE: Lazy<DashMap<String, (ResolvedAccount, Instant)>> = Lazy::new(DashMap::new);
const TTL: Duration = Duration::from_secs(60 * 10); // 10 minutes

pub struct BankAccountService;

impl BankAccountService {
    pub async fn list_user_accounts(
        state: &AppState,
        claims: &Claims,
    ) -> Result<BankAccountsResponse, ApiError> {
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            warn!("bank_accounts.list: invalid subject in token");
            ApiError::Auth(AuthError::InvalidToken("Invalid token".into()))
        })?;

        let mut conn = state.db.get().map_err(|_| {
            error!("bank_accounts.list: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let accounts =
            BankAccountRepository::find_all_by_user(&mut conn, user_id).map_err(|_| {
                error!("bank_accounts.list: query failed");
                ApiError::Internal("Failed to fetch bank accounts".into())
            })?;

        Ok(BankAccountsResponse {
            bank_accounts: accounts
                .into_iter()
                .map(BankAccountResponse::from)
                .collect(),
        })
    }

    pub async fn get_bank_accounts(
        state: &AppState,
        user_id_val: Uuid,
    ) -> Result<Vec<BankAccount>, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e: r2d2::Error| ApiError::DatabaseConnection(e.to_string()))?;

        let accounts = BankAccountRepository::find_all_by_user(&mut conn, user_id_val)?;

        Ok(accounts)
    }

    pub async fn create_bank_account(
        state: &AppState,
        user_id_val: Uuid,
        req: BankRequest,
    ) -> Result<BankAccount, ApiError> {

        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // idempotency check using Repository
        if let Some(existing) = BankAccountRepository::find_active_by_details(
            &mut conn,
            user_id_val,
            &req.bank_code,
            &req.account_number,
        )? {
            return Ok(existing);
        }

        let account_details =
            Self::resolve_account_details(state, &req.bank_code, &req.account_number).await?;

        let account_name = account_details.account_name;

        //make call to paystack via its client
        let paystack_client = PaystackClient::new(
            state.http_client.clone(),
            &state.config.paystack_details.paystack_api_url,
            state.config.paystack_details.paystack_secret_key.clone(),
        )?;

        let payload = PaystackClient::create_recipient(
            &account_name,
            &req.account_number,
            &req.bank_code,
            CurrencyCode::NGN,
        );

        let recipient_code = paystack_client
            .create_transfer_recipient(payload)
            .await
            .map_err(|_| ApiError::Payment("Unable to create transfer recipient".into()))?;

        let new_account = NewBankAccount {
            user_id: user_id_val,
            bank_name: Some(&req.bank_name),
            account_number: &req.account_number,
            account_name: Some(&account_name),
            bank_code: &req.bank_code,
            provider_recipient_id: Some(&recipient_code),
            is_verified: true,
        };

        BankAccountRepository::create(&mut conn, new_account)
    }

    pub async fn resolve_account_details(
        state: &AppState,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ResolvedAccount, ApiError> {
        Self::validate_bank_details(bank_code, account_number)?;

        let cache_key = format!("{bank_code}:{account_number}");

        if let Some(cached) = Self::get(&cache_key) {
            info!("Bank resolve cache hit: {}", cache_key);
            return Ok(cached);
        }

        let mut url = Url::parse(&state.config.paystack_details.paystack_api_url)
            .map_err(|_| ApiError::Internal("Invalid Paystack base URL".into()))?;

        url.set_path("bank/resolve");
        url.query_pairs_mut()
            .append_pair("account_number", account_number)
            .append_pair("bank_code", bank_code);

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
            .header("User-Agent", "Payego/1.0")
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to reach Paystack during account resolution");
                ApiError::Payment(format!("Paystack service unavailable: {}", e))
            })?;

        let status = resp.status();
        let body_text = resp.text().await.map_err(|e| {
            error!(error = %e, "Failed to read Paystack response body");
            ApiError::Payment("Failed to read Paystack response".into())
        })?;

        if !status.is_success() {
            error!(
                status = %status,
                body = %body_text,
                "Paystack account resolution failed"
            );
            return Err(ApiError::Payment(format!(
                "Paystack error {}: {}",
                status, body_text
            )));
        }

        let body: PaystackResolveResponse = serde_json::from_str(&body_text).map_err(|e| {
            error!(error = %e, body = %body_text, "Failed to parse Paystack resolution response");
            ApiError::Payment("Invalid response from Paystack".into())
        })?;

        if !body.status {
            tracing::log::warn!("Paystack resolve failed: {}", body.message);
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

    pub fn validate_bank_details(bank_code: &str, account_number: &str) -> Result<(), ApiError> {
        if !ACCOUNT_NUMBER_RE
            .as_ref()
            .map_err(|_| ApiError::Internal("Account number regex misconfigured".into()))?
            .is_match(account_number)
        {
            error!("Account number must be 10 digits");
            return Err(ApiError::BadRequest(
                "Account number must be 10 digits".to_string(),
            ));
        }

        if !BANK_CODE_RE
            .as_ref()
            .map_err(|_| ApiError::Internal("Account number regex misconfigured".into()))?
            .is_match(bank_code)
        {
            error!("Bank code must be 3–10 digits");
            return Err(ApiError::BadRequest(
                "Bank code must be 3–10 digits".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn delete_bank_account(
        state: &AppState,
        user_id: Uuid,
        bank_account_id: Uuid,
    ) -> Result<DeleteResponse, ApiError> {
        let mut conn = state.db.get().map_err(|e| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        BankAccountRepository::delete_by_id_and_user(&mut conn, bank_account_id, user_id)?;

        Ok(DeleteResponse {
            account_id: bank_account_id,
            message: "Bank account deleted successfully".into(),
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
