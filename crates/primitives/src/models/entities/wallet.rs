use crate::models::entities::enum_types::CurrencyCode;
use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = crate::schema::wallets)]
#[diesel(belongs_to(crate::models::entities::user::User))]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub currency: CurrencyCode,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::wallets)]
pub struct NewWallet {
    pub user_id: Uuid,
    pub currency: CurrencyCode,
}
