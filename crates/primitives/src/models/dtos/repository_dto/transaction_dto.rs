use diesel::Queryable;
use uuid::Uuid;
use crate::models::enum_types::{CurrencyCode, PaymentState, TransactionIntent};

#[derive(Queryable)]
pub struct RecentUser {
    pub id: Uuid,
    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub txn_state: PaymentState,
    pub reference: Uuid
}


