use crate::models::entities::enum_types::{
    CurrencyCode, PaymentProvider, PaymentState, TransactionIntent,
};
use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(belongs_to(crate::models::entities::user::User))]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub counterparty_id: Option<Uuid>,

    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,

    pub txn_state: PaymentState,
    pub provider: Option<PaymentProvider>,
    pub provider_reference: Option<String>,

    pub idempotency_key: String,
    pub reference: Uuid,

    pub description: Option<String>,
    pub metadata: Value,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transactions)]
pub struct NewTransaction<'a> {
    pub user_id: Uuid,
    pub counterparty_id: Option<Uuid>,
    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,
    pub txn_state: PaymentState,
    pub provider: Option<PaymentProvider>,
    pub provider_reference: Option<&'a str>,
    pub idempotency_key: &'a str,
    pub reference: Uuid,
    pub description: Option<&'a str>,
    pub metadata: Value,
}





