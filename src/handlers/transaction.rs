use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::handlers::user_bank_accounts::Account;
use crate::models::models::AppState;
use crate::schema::transactions::{reference, user_id};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::response::IntoResponse;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::handlers::transfer_internal::TransferRequest;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::transactions)]
pub struct Transaction {
    pub amount: i64, // BIGINT for cents, can be negative for debits
    pub transaction_type: String,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct TransactionResponse {
    id: String,
    // #[serde(rename = "type")]
    transaction_type: String,
    amount: i64, // Keep as cents, let frontend format
    currency: String,
    created_at: String, // ISO 8601 format
    status: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TransactionRequest {
    txn_id: String,
}

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
) -> Result<Json<TransactionResponse>, (StatusCode, String)> {

    let usr_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID in JWT: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;
    let txn_id = Uuid::parse_str(&transaction_id).map_err(|e: uuid::Error| {
        error!("Invalid tnx id: {}", e);
        ApiError::Auth("Invalid transaction ID".to_string())
    })?;

    let conn = &mut state.db.get().map_err(|e: diesel::r2d2::PoolError| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let transact = crate::schema::transactions::table
        .filter(crate::schema::transactions::reference.eq(txn_id))
        .filter(crate::schema::transactions::user_id.eq(usr_id))
        .select(Transaction::as_select())
        .first::<Transaction>(conn)
        .map_err(|e: diesel::result::Error| {
            error!("Transaction query failed: {}", e);
            ApiError::Database(e)
        })?;

    Ok(Json(TransactionResponse {
        id: transaction_id,
        transaction_type: transact.transaction_type,
        amount: transact.amount, // In cents
        currency: transact.currency,
        created_at: transact.created_at.to_rfc3339(),
        status: transact.status,
        notes: transact.description,
    }))
}
