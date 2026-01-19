use axum::{
    extract::{Extension, Path, State},
    Json,
};
use diesel::prelude::*;
use payego_core::services::transaction_service::{ TransactionService};
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use uuid::Uuid;
use payego_primitives::models::transaction_dto::TransactionResponse;

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
    
    let tx = TransactionService::get_user_transaction(&state, &claims, transaction_id).await?;

    Ok(Json(tx))
}
