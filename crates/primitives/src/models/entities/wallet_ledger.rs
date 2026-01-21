use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = crate::schema::wallet_ledger)]
#[diesel(belongs_to(crate::models::entities::wallet::Wallet))]
#[diesel(belongs_to(crate::models::entities::transaction::Transaction))]
pub struct WalletLedger {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::wallet_ledger)]
pub struct NewWalletLedger {
    pub wallet_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
}
