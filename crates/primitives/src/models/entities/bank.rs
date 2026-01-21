use crate::models::entities::enum_types::CurrencyCode;
use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, ToSchema, Deserialize)]
#[diesel(table_name = crate::schema::banks)]
pub struct Bank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: CurrencyCode,
    pub country: String,
    pub is_active: bool,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::banks)]
pub struct NewBank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: CurrencyCode,
    pub country: String,
    pub is_active: bool,
}

#[derive(Debug, Queryable, Identifiable, Associations, ToSchema, Serialize)]
#[diesel(table_name = crate::schema::bank_accounts)]
#[diesel(belongs_to(crate::models::entities::user::User))]
pub struct BankAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
    pub provider_recipient_id: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::bank_accounts)]
pub struct NewBankAccount<'a> {
    pub user_id: Uuid,
    pub bank_code: &'a str,
    pub account_number: &'a str,
    pub account_name: Option<&'a str>,
    pub bank_name: Option<&'a str>,
    pub provider_recipient_id: Option<&'a str>,
    pub is_verified: bool,
}
