use axum::{
    extract::{Extension, State},
    Json,
};
use payego_core::services::bank_account_service::{
    ApiError, AppState, BankAccountService, BankAccountsResponse, Claims,
};
use std::sync::Arc;

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
