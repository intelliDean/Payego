use axum::{
    extract::{State, Extension},
    Json,
    http::StatusCode,
};
use diesel::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{error, info};
use utoipa::ToSchema;
use crate::{AppState, error::ApiError};
use crate::config::security_config::Claims;
use crate::schema::bank_accounts;


#[derive(Queryable, Selectable)]
#[diesel(table_name = bank_accounts)]
pub struct BankAccount {
    // #[schema(value_type = String)]
    pub id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>
}

#[derive(Serialize, ToSchema)]
pub struct Account {
    pub id: String,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>
}

#[derive(Serialize, ToSchema)]
pub struct BankResponse {
    pub bank_accounts: Vec<Account>,
}

#[utoipa::path(
    get,
    path = "/api/bank_accounts",
    responses(
        (status = 200, description = "List of user wallets", body = BankResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Bank Accounts"
)]
pub async fn user_bank_accounts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<BankResponse>, (StatusCode, String)> {

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let bank_accounts = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .select(BankAccount::as_select())
        .load(conn)
        .map_err(|e| {
            error!("Failed to load wallets: {}", e);
            ApiError::Database(e)
        })?
        .into_iter()
        .map(|account| Account {
            id: account.id.to_string(),
            bank_code: account.bank_code,
            account_number: account.account_number,
            account_name: account.account_name,
            bank_name: account.bank_name,
        })
        .collect::<Vec<Account>>();

    info!("Fetched {} banks accounts for user_id={}", bank_accounts.len(), user_id);

    Ok(Json(BankResponse { bank_accounts }))
}