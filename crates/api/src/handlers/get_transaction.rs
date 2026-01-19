use axum::{extract::State, Extension, Json};
use payego_core::services::transaction_service::{TransactionService};
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use payego_primitives::models::transaction_dto::TransactionsResponse;

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
    
    let response = 
        TransactionService::recent_transactions(&state, claims.user_id()?).await?;

    Ok(Json(response))
}
