use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::schema::bank_accounts;
use payego_primitives::{error::{ApiError, AuthError}, models::AppState};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = bank_accounts)]
pub struct BankAccount {
    // #[schema(value_type = String)]
    pub id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Account {
    pub id: String,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
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
) -> Result<Json<BankResponse>, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth(AuthError::InvalidToken("Invalid user ID".to_string()))
    })?;

    let conn = &mut state.db.get().map_err(|e: r2d2::Error| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let bank_accounts = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .select(BankAccount::as_select())
        .load(conn)
        .map_err(|e| {
            error!("Failed to load wallets: {}", e);
            ApiError::from(e)
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

    info!(
        "Fetched {} banks accounts for user_id={}",
        bank_accounts.len(),
        user_id
    );

    Ok(Json(BankResponse { bank_accounts }))
}
