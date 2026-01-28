pub use crate::app_state::AppState;
pub use crate::security::Claims;
use crate::repositories::bank_account_repository::BankAccountRepository;
use crate::clients::paystack::PaystackClient;
use dashmap::DashMap;
use once_cell::sync::Lazy;
pub use payego_primitives::{
    error::ApiError,
    models::{
        bank::{BankAccount, NewBankAccount},
        dtos::bank_dto::{
            BankAccountResponse, BankAccountsResponse, BankRequest, DeleteResponse,
            ResolveAccountRequest, ResolveAccountResponse, ResolvedAccount,
        },
        enum_types::CurrencyCode,
    },
};
use regex::Regex;
use std::time::{Duration, Instant};
use tracing::{error, info};
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
        let user_id = claims.user_id()?;

        let mut conn = state.db.get().map_err(|e| {
            error!(error = %e, "Failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let accounts = BankAccountRepository::find_all_by_user(&mut conn, user_id)?;

        Ok(BankAccountsResponse {
            bank_accounts: accounts
                .into_iter()
                .map(BankAccountResponse::from)
                .collect(),
        })
    }

    pub async fn create_bank_account(
        state: &AppState,
        user_id_val: Uuid,
        req: BankRequest,
    ) -> Result<BankAccount, ApiError> {
        let mut conn = state.db.get().map_err(|e| {
            error!(error = %e, "Database error");
            ApiError::DatabaseConnection(e.to_string())
        })?;

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

        let payload = PaystackClient::create_recipient_payload(
            &account_name,
            &req.account_number,
            &req.bank_code,
            CurrencyCode::NGN,
        );

        let recipient_code = state.paystack
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

        let account = BankAccountRepository::create(&mut conn, new_account)?;

        info!(
            user_id = %user_id_val,
            account_id = %account.id,
            "Bank account created successfully"
        );

        Ok(account)
    }

    pub async fn resolve_account_details(
        state: &AppState,
        bank_code: &str,
        account_number: &str,
    ) -> Result<ResolvedAccount, ApiError> {
        Self::validate_bank_details(bank_code, account_number)?;

        let cache_key = format!("{bank_code}:{account_number}");

        if let Some(cached) = Self::get(&cache_key) {
            return Ok(cached);
        }

        let body = state.paystack.resolve_bank_account(account_number, bank_code).await?;

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
            return Err(ApiError::BadRequest(
                "Account number must be 10 digits".to_string(),
            ));
        }

        if !BANK_CODE_RE
            .as_ref()
            .map_err(|_| ApiError::Internal("Bank code regex misconfigured".into()))?
            .is_match(bank_code)
        {
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
            error!(error = %e, "Database error");
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
