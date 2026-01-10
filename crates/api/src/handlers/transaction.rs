use axum::{
    extract::{Extension, Path, State},
    Json,
};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, Transaction, TransactionResponse};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/transactions/{transaction_id}",
    params(
        ("transaction_id" = String, Path, description = "UUID of the transaction (reference)")
    ),
    responses(
        (status = 200, description = "Transaction details", body = TransactionResponse),
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
    Path(transaction_id): Path<String>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let usr_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID in JWT: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;
    let txn_id = Uuid::parse_str(&transaction_id).map_err(|e: uuid::Error| {
        error!("Invalid tnx id: {}", e);
        ApiError::Auth("Invalid transaction ID".to_string())
    })?;

    let mut conn = state.db.get().map_err(|e: r2d2::Error| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let transact = payego_primitives::schema::transactions::table
        .filter(payego_primitives::schema::transactions::reference.eq(txn_id))
        .filter(payego_primitives::schema::transactions::user_id.eq(usr_id))
        .first::<Transaction>(&mut conn)
        .map_err(|e: diesel::result::Error| {
            error!("Transaction query failed: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(TransactionResponse {
        id: transaction_id,
        transaction_type: transact.transaction_type,
        amount: transact.amount,
        currency: transact.currency,
        created_at: transact.created_at.to_rfc3339(),
        status: transact.status,
        notes: transact.description,
    }))
}
