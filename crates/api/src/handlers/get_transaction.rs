use axum::{extract::State, Extension, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use http::StatusCode;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::schema::transactions;
use serde::Serialize;
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use payego_primitives::models::enum_types::{CurrencyCode, PaymentState, TransactionIntent};



#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionRow>,
}

#[derive(Queryable, Debug, Serialize, ToSchema)]
struct TransactionRow {
    pub id: Uuid,
    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,
    pub created_at: DateTime<Utc>,
    pub txn_state: PaymentState,
}


const RECENT_TX_LIMIT: i64 = 5;

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
    let usr_id = claims.user_id()?;

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
            intent,
            amount,
            currency,
            created_at,
            txn_state,
        ))
        .load::<TransactionRow>(&mut conn)
        .map_err(ApiError::from)?;

    

    Ok(Json(TransactionsResponse { transactions: rows  }))
}





