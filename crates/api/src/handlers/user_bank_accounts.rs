use crate::config::swagger_config::ApiErrorResponse;
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
    path = "/api/user/banks",
    tag = "Bank",
    summary = "List all linked bank accounts",
    description = "Retrieves a list of bank accounts previously linked/added by the authenticated user. \
                   Returns account details such as account number (masked), bank name, account name, \
                   whether it's the default/withdrawal account, verification status, and creation timestamp. \
                   Useful for displaying withdrawal options, account selection during transfers, \
                   or allowing users to manage (remove/set default) their bank accounts. \
                   Only returns accounts belonging to the current authenticated user.",
    operation_id = "listUserBankAccounts",
    responses(
        ( status = 200, description = "Successfully retrieved list of linked bank accounts", body = BankAccountsResponse),
        ( status = 401, description = "Unauthorized – missing, invalid, or expired authentication token", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden – insufficient permissions (rare, but possible with role-based restrictions)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests – rate limit exceeded for account listing", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error – failed to retrieve bank accounts", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn user_bank_accounts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<BankAccountsResponse>, ApiError> {
    let accounts = BankAccountService::list_user_accounts(&state, &claims).await?;
    Ok(Json(accounts))
}
