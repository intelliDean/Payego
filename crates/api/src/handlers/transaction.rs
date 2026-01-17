use axum::{
    extract::{Extension, Path, State},
    Json,
};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use std::sync::Arc;
use serde::Serialize;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::enum_types::{CurrencyCode, PaymentState, TransactionIntent};
use payego_primitives::models::transaction::Transaction;
use payego_primitives::schema::transactions;

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub id: String,
    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,
    pub status: PaymentState,
    pub created_at: String,
    pub description: Option<String>,
}


impl From<Transaction> for TransactionResponse {
    fn from(tx: Transaction) -> Self {
        Self {
            id: tx.reference.to_string(),
            intent: tx.intent,
            amount: tx.amount,
            currency: tx.currency,
            status: tx.txn_state,
            created_at: tx.created_at.to_rfc3339(),
            description: tx.description,
        }
    }
}


#[utoipa::path(
    get,
    path = "/api/transactions/{transaction_id}",
    params(
        ("transaction_id" = Uuid, Path, description = "Transaction reference UUID")
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
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        ApiError::Auth(AuthError::InvalidToken("Invalid user ID".into()))
    })?;

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let transaction = transactions::table
        .filter(transactions::reference.eq(transaction_id))
        .filter(transactions::user_id.eq(user_id))
        .first::<Transaction>(&mut conn)
        .optional()
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::Internal("Transaction not found".into()))?;

    Ok(Json(transaction.into()))
}