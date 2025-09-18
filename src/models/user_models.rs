use crate::schema::*;
use crate::utility::validate_password;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Queryable, Insertable, Selectable, Identifiable)]
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

// #[derive(Queryable, Selectable, Serialize, Deserialize)]
// #[diesel(table_name = crate::schema::users)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct User {
//     pub id: Uuid,
//     pub email: String,
//     pub password_hash: String,
//     pub username: Option<String>,
//     pub created_at: chrono::DateTime<chrono::Utc>,
//     pub updated_at: chrono::DateTime<chrono::Utc>,
// }

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub username: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: NewUser,
}

// Wallets table
#[derive(Queryable, Insertable, Selectable, Identifiable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: i64, // BIGINT for cents (e.g., 100 = $1.00)
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = wallets)]
pub struct NewWallet {
    pub user_id: Uuid,
    pub balance: i64,
    pub currency: String,
}

// Transactions table
#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub amount: i64, // BIGINT for cents, can be negative for debits
    pub transaction_type: String,
    pub status: String,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub user_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub amount: i64,
    pub transaction_type: String,
    pub status: String,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub reference: Uuid,
    // pub metadata: Option<JsonValue>,
}

// Bank Accounts table
#[derive(Queryable, Insertable, Selectable, Identifiable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = bank_accounts)]
pub struct BankAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub paystack_recipient_code: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = bank_accounts)]
pub struct NewBankAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub paystack_recipient_code: Option<String>,
    pub is_verified: bool,
}

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub(crate) email: String,
    #[validate(length( min = 8), custom(function = "validate_password"))]
    pub(crate) password: String,
    #[validate(length(
        min = 3,
        max = 100,
        message = "Username must be between 3 and 100 characters"
    ))]
    pub(crate) username: Option<String>,
}


#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub message: String,
    pub username: Option<String>,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub(crate) email: String,
    #[validate(length( min = 8), custom(function = "validate_password"))]
    pub(crate) password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub(crate) token: String,
    pub(crate) user_email: String,
}
