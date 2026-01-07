use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::models::models::AppState;
use crate::schema::transactions;
use axum::{extract::State, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use headers::authorization::Bearer;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: Uuid,
    transaction_type: String,
    amount: i64,
    currency: String,
    created_at: DateTime<Utc>,
    status: String,
}

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
) -> Result<(StatusCode, Json<TransactionsResponse>), (StatusCode, String)> {
    // Validate JWT token
    let usr_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch last 5 transactions
    use crate::schema::transactions::dsl::*;
    let results = transactions
        .filter(user_id.eq(user_id))
        .order(created_at.desc())
        .limit(5)
        .select(Transaction::as_select())
        .load(conn)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch transactions".to_string(),
            )
        })?
        .into_iter()
        .map(|transact| Transact {
            id: transact.id.to_string(),
            transaction_type: transact.transaction_type,
            amount: transact.amount,
            currency: transact.currency,
            created_at: transact.created_at.to_string(),
            status: transact.status,
        })
        .collect::<Vec<Transact>>();

    Ok((
        StatusCode::OK,
        Json(TransactionsResponse {
            transactions: results,
        }),
    ))
}
