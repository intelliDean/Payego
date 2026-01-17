use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize}; // Kept for mixed usage if needed, but aiming for separation
use serde_json::{Value};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::CurrencyCode"]
pub enum CurrencyCode {
    USD, NGN, GBP, EUR, CAD, AUD, CHF, JPY, CNY, SEK, NZD, 
    MXN, SGD, HKD, NOK, KRW, TRY, INR, BRL, ZAR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TransactionIntent"]
pub enum TransactionIntent {
    TopUp,
    Payout,
    Transfer,
    Conversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentState"]
pub enum PaymentState {
    Pending,
    RequiresAction,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentProvider"]
pub enum PaymentProvider {
    Stripe,
    Paypal,
    Paystack,
    Internal,
}



//======== USER ===========
#[derive(
    Debug, Clone, Queryable, Identifiable, Serialize
)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
    pub username: Option<&'a str>,
}


//======= WALLET ============
#[derive(
    Debug, Clone, Queryable, Identifiable, Associations, Serialize
)]
#[diesel(table_name = crate::schema::wallets)]
#[diesel(belongs_to(crate::models::entities::User))]
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

//========= TRANSACTION ============
#[derive(
    Debug, Clone, Queryable, Identifiable, Associations, Serialize
)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(belongs_to(crate::models::entities::User))]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub counterparty_id: Option<Uuid>,

    pub intent: TransactionIntent,
    pub amount: i64,
    pub currency: CurrencyCode,

    pub payment_state: PaymentState,
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
pub struct NewTransaction {
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
}

//========== WALLET LEDGER ===============
#[derive(
    Debug, Clone, Queryable, Identifiable, Associations, Serialize
)]
#[diesel(table_name = crate::schema::wallet_ledger)]
#[diesel(belongs_to(crate::models::entities::Wallet))]
#[diesel(belongs_to(crate::models::entities::Transaction))]
pub struct WalletLedgerEntry {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::wallet_ledger)]
pub struct NewWalletLedgerEntry {
    pub wallet_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
}

//========= BANK =================
#[derive(Debug, Queryable, Identifiable, Serialize)]
#[diesel(table_name = crate::schema::banks)]
pub struct Bank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: CurrencyCode,
    pub country: String,
    pub is_active: bool,
}
#[derive(
    Debug, Queryable, Identifiable, Associations, Serialize
)]
#[diesel(table_name = crate::schema::bank_accounts)]
#[diesel(belongs_to(crate::models::entities::User))]
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

//===== AUTH TABLES ============
#[derive(Queryable, Identifiable)]
#[diesel(table_name = crate::schema::blacklisted_tokens)]
#[diesel(primary_key(jti))]
pub struct BlacklistedToken {
    pub jti: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(table_name = crate::schema::refresh_tokens)]
#[diesel(belongs_to(crate::models::entities::User))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}




//=============================
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
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
    pub username: Option<&'a str>,
}

#[derive(Queryable, Insertable, Identifiable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = crate::schema::banks)]
pub struct Bank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: String,
    pub country: String,
    pub is_active: bool,
}

#[derive(Queryable, Insertable, Selectable, Identifiable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub currency: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = wallets)]
pub struct NewWallet<'a> {
    pub user_id: Uuid,
    pub balance: i64,
    pub currency: &'a str,
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = wallet_ledger)]
pub struct WalletLedger {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = wallet_ledger)]
pub struct NewWalletLedger {
    pub wallet_id: Uuid,
    pub transaction_id: Uuid,
    pub amount: i64,
}

// Transactions
#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub counterparty_id: Option<Uuid>,

    pub intent: String,

    pub amount: i64,
    pub currency: String,

    pub payment_state: String,
    pub provider: Option<String>,
    pub provider_reference: Option<String>,

    pub idempotency_key: String,
    pub reference: Uuid,

    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction<'a> {
    pub user_id: Uuid,
    pub counterparty_id: Option<Uuid>,

    pub intent: &'a str,

    pub amount: i64,
    pub currency: &'a str,

    pub payment_state: &'a str,
    pub provider: Option<&'a str>,
    pub provider_reference: Option<&'a str>,

    pub idempotency_key: &'a str,
    pub reference: Uuid,

    pub description: Option<&'a str>,
    pub metadata: Option<serde_json::Value>,
}

// Bank Accounts
#[derive(
    Queryable,
    Insertable,
    Selectable,
    Identifiable,
    Debug,
    Clone,
    serde::Serialize,
    utoipa::ToSchema,
)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = bank_accounts)]
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

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = bank_accounts)]
pub struct NewBankAccount<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_code: &'a str,
    pub account_number: &'a str,
    pub account_name: Option<&'a str>,
    pub bank_name: Option<&'a str>,
    pub provider_recipient_id: Option<&'a str>,
    pub is_verified: bool,
}

// Refresh Tokens
// #[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
// #[diesel(table_name = refresh_tokens)]
// pub struct RefreshToken {
//     pub id: Uuid,
//     pub user_id: Uuid,
//     pub token_hash: String,
//     pub expires_at: DateTime<Utc>,
//     pub revoked: bool,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshToken<'a> {
    pub user_id: Uuid,
    pub token_hash: &'a str,
    pub expires_at: DateTime<Utc>,
}
