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
) -> Result<(StatusCode, Json<TransactionsResponse>), ApiError> {
    // Validate JWT token
    let usr_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth(AuthError::InvalidToken("Invalid user ID".to_string()))
    })?;

    let conn = &mut state.db.get().map_err(|e: r2d2::Error| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch last 5 transactions
    use payego_primitives::schema::transactions::dsl::*;
    let results = transactions
        .filter(user_id.eq(usr_id))
        .order(created_at.desc())
        .limit(5)
        .select(Transaction::as_select())
        .load(conn)
        .map_err(ApiError::from)?
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
