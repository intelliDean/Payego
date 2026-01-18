use diesel::prelude::*;
use serde::Serialize;
use tracing::{error, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::app_state::AppState,
        bank::BankAccount,
    },
    schema::bank_accounts,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct BankAccountResponse {
    pub id: String,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
    pub is_verified: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankAccountsResponse {
    pub bank_accounts: Vec<BankAccountResponse>,
}

impl From<BankAccount> for BankAccountResponse {
    fn from(account: BankAccount) -> Self {
        Self {
            id: account.id.to_string(),
            bank_code: account.bank_code,
            account_number: account.account_number,
            account_name: account.account_name,
            bank_name: account.bank_name,
            is_verified: account.is_verified,
        }
    }
}
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
}
