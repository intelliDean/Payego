use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize}; // Kept for mixed usage if needed, but aiming for separation
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Insertable, Selectable, Identifiable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub username: Option<String>,
}

#[derive(
    Queryable, Insertable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize, ToSchema,
)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = crate::schema::banks)]
pub struct Bank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: String,
    pub country: String,
    pub gateway: Option<String>,
    pub pay_with_bank: Option<bool>,
    pub is_active: Option<bool>,
}

// Wallets
#[derive(Queryable, Insertable, Selectable, Identifiable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: i64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = wallets)]
pub struct NewWallet {
    pub user_id: Uuid,
    pub balance: i64,
    pub currency: String,
}

// Transactions
#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub amount: i64,
    pub transaction_type: String,
    pub currency: String,
    pub status: String,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub reference: Uuid,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub user_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub amount: i64,
    pub transaction_type: String,
    pub currency: String,
    pub status: String,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub reference: Uuid,
    pub metadata: Option<JsonValue>,
}

// Bank Accounts
#[derive(Queryable, Insertable, Selectable, Identifiable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = bank_accounts)]
pub struct BankAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
    pub paystack_recipient_code: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = bank_accounts)]
pub struct NewBankAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
    pub paystack_recipient_code: Option<String>,
    pub is_verified: bool,
}

// Refresh Tokens
#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}
