use crate::models::enum_types::{CurrencyCode, PaymentState, TransactionIntent};
use crate::models::transaction::Transaction;
use chrono::{DateTime, Utc};
use diesel::Queryable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, ToSchema)]
pub struct TransactionSummaryDto {
    pub id: Uuid,
    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,
    pub created_at: DateTime<Utc>,
    pub status: PaymentState,
    pub reference: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionSummaryDto>,
}

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExchangeQuery {
    pub from: CurrencyCode,
    pub to: CurrencyCode,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExchangeResponse {
    pub from: String,
    pub to: String,
    pub rate: f64,
}
