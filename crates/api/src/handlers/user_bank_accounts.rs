use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::schema::bank_accounts;
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::bank::BankAccount;

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

#[utoipa::path(
    get,
    path = "/api/bank_accounts",
    responses(
        (status = 200, description = "List of user bank accounts", body = BankAccountsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Bank Accounts"
)]
pub async fn user_bank_accounts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<BankAccountsResponse>, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Auth(AuthError::InvalidToken("Invalid user ID".into())))?;

    let mut conn = state.db.get().map_err(|e| {
        error!("DB connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let accounts = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .order(bank_accounts::created_at.desc())
        .load::<BankAccount>(&mut conn)
        .map_err(ApiError::from)?;

    let response = accounts
        .into_iter()
        .map(BankAccountResponse::from)
        .collect::<Vec<_>>();

    info!(
        "Fetched {} bank accounts for user_id={}",
        response.len(),
        user_id
    );

    Ok(Json(BankAccountsResponse {
        bank_accounts: response,
    }))
}
