use payego_primitives::error::ApiErrorResponse;
use axum::{extract::State, Extension, Json};
use payego_core::services::transaction_service::{
    ApiError, AppState, Claims, TransactionService, TransactionsResponse,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/user/transactions",
    summary = "Get list of user transactions",
    description = "Retrieves a paginated list of the authenticated user's transaction history. \
                   Includes deposits, withdrawals, internal transfers, external transfers, \
                   top-ups, payments, and other wallet activities. \
                   Results are ordered by creation date (newest first). \
                   Supports filtering and pagination via query parameters.",
    operation_id = "getUserTransactions",
    tags = ["Transactions"],

    responses(
        (status = 200,description = "Successfully retrieved paginated list of transactions",body = TransactionsResponse),
        (status = 400,description = "Bad request – invalid query parameters (e.g. limit > 100, invalid date format)",body = ApiErrorResponse),
        (status = 401,description = "Unauthorized – missing, invalid, or expired authentication token",body = ApiErrorResponse),
        (status = 429,description = "Too many requests – rate limit exceeded",body = ApiErrorResponse),
        (status = 500,description = "Internal server error – failed to retrieve transactions",body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<TransactionsResponse>, ApiError> {
    let user_id = claims.user_id()?;

    let response = TransactionService::recent_transactions(&state, user_id).await?;

    Ok(Json(response))
}
