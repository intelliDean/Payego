use diesel::prelude::*;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::Serialize;
use serde_json::json;
use tracing::{error, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::services::bank_service::BankService;
use payego_primitives::models::bank::NewBankAccount;
use payego_primitives::models::bank_dtos::{
    BankAccountResponse, BankAccountsResponse, BankRequest,
};
use payego_primitives::models::dtos::bank_dtos::PaystackRecipientResponse;
use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{app_state::app_state::AppState, bank::BankAccount},
    schema::bank_accounts,
};

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

        let accounts = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id))
            .order(bank_accounts::created_at.desc())
            .load::<BankAccount>(&mut conn)
            .map_err(|_| {
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

        let accounts = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id_val))
            .load::<BankAccount>(&mut conn)
            .map_err(ApiError::from)?;

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
            BankService::resolve_account_details(state, &req.bank_code, &req.account_number)
                .await?;

        let account_name = account_details.account_name;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|_| ApiError::Internal("HTTP client error".into()))?;

        let resp = client
            .post(format!(
                "{}/transferrecipient",
                state.config.paystack_details.paystack_api_url
            ))
            .bearer_auth(
                state
                    .config
                    .paystack_details
                    .paystack_secret_key
                    .expose_secret(),
            )
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
}
