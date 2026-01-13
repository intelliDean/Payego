use axum::{extract::State, Extension, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use http::StatusCode;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::AppState;
use payego_primitives::schema::transactions;
use serde::Serialize;
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;


#[derive(Debug, Serialize, ToSchema)]
pub struct Transact {
    id: String,
    #[serde(rename = "type")]
    transaction_type: String,
    amount: i64,
    currency: String,
    created_at: String,
    status: String,
}

#[derive(Serialize, ToSchema)]
pub struct TransactionsResponse {
    transactions: Vec<Transact>,
}

const RECENT_TX_LIMIT: i64 = 5;

#[derive(Queryable)]
struct TransactionRow {
    id: Uuid,
    transaction_type: String,
    amount: i64,
    currency: String,
    created_at: DateTime<Utc>,
    status: String,
}

#[utoipa::path(
    get,
    path = "/api/transactions",
    responses(
        (status = 200, description = "List of recent transactions", body = TransactionsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Transactions",
    security(("bearerAuth" = [])),
)]
pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<TransactionsResponse>), ApiError> {

    let usr_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        ApiError::Auth(AuthError::InvalidToken("Invalid subject".into()))
    })?;

    let mut conn = state.db.get().map_err(|e| {
        error!("DB connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    use payego_primitives::schema::transactions::dsl::*;

    let rows = transactions
        .filter(user_id.eq(usr_id))
        .order(created_at.desc())
        .limit(RECENT_TX_LIMIT)
        .select((
            id,
            transaction_type,
            amount,
            currency,
            created_at,
            status,
        ))
        .load::<TransactionRow>(&mut conn)
        .map_err(ApiError::from)?;

    let response = rows
        .into_iter()
        .map(|tx| Transact {
            id: tx.id.to_string(),
            transaction_type: tx.transaction_type,
            amount: tx.amount,
            currency: tx.currency,
            created_at: tx.created_at.to_rfc3339(),
            status: tx.status,
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(TransactionsResponse {
            transactions: response,
        }),
    ))
}