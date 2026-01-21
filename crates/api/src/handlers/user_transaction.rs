use crate::config::swagger_config::ApiErrorResponse;
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use payego_core::services::transaction_service::{
    ApiError, AppState, Claims, TransactionResponse, TransactionService,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/transactions/{transaction_id}",
    tag = "Transactions",
    summary = "Get details of a specific transaction",
    description = "Retrieves complete details of a single transaction by its unique UUID reference. \
                   Includes transaction type (top-up, withdrawal, transfer, conversion, payment, etc.), \
                   amount, currency, status, timestamps, payment provider reference (if applicable), \
                   fees, and related metadata. \
                   Only transactions belonging to the authenticated user are accessible. \
                   Use this endpoint for transaction receipts, status polling, or detailed history views.",
    operation_id = "getTransactionById",
    responses(
        ( status = 200, description = "Transaction details retrieved successfully", body = TransactionResponse),
        ( status = 400, description = "Bad request – invalid transaction ID format (not a valid UUID)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized – missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden – transaction does not belong to the authenticated user", body = ApiErrorResponse),
        ( status = 404, description = "Not found – no transaction exists with the provided ID", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests – rate limit exceeded for transaction lookups", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error – failed to retrieve transaction details", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn get_user_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let tnx_response =
        TransactionService::get_user_transaction(&state, &claims, transaction_id).await?;

    Ok(Json(tnx_response))
}
