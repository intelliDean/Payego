use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_core::services::bank_account_service::{BankAccountService};
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use payego_primitives::models::bank_dtos::BankAccountsResponse;

#[utoipa::path(
    get,
    path = "/api/bank_accounts",
    responses(
        (status = 200, description = "List of user bank accounts"),
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

    let accounts = BankAccountService::list_user_accounts(&state, &claims).await?;
    Ok(Json(accounts))
}
