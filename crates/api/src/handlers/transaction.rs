use axum::{
    extract::{Extension, Path, State},
    Json,
};
use payego_core::services::transaction_service::{
    TransactionService, ApiError, AppState, TransactionResponse, Claims
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/transactions/{transaction_id}",
    params(
        ("transaction_id" = Uuid, Path, description = "Transaction reference UUID")
    ),
    responses(
        (status = 200, description = "Transaction details"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Transaction not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Transaction"
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
