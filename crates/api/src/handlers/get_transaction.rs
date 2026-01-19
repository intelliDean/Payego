use axum::{extract::State, Extension, Json};
use payego_core::services::transaction_service::{
    ApiError, AppState, TransactionService, TransactionsResponse, Claims
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/transactions",
    responses(
        (status = 200, body = TransactionsResponse),
        (status = 401),
        (status = 500)
    ),
    tag = "Transactions",
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
