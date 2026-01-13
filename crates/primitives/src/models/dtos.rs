
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// --- BANK DTOS ---
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct BankRequest {
    pub bank_name: String,
    pub account_number: String,
    pub bank_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankResponse {
    pub id: Uuid,
    pub bank_name: String,
    pub account_number: String,
    pub account_name: String,
}

// --- CONVERSION DTOS ---
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ConvertRequest {
    pub from_currency: String,
    pub to_currency: String,
    pub amount: f64,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConvertResponse {
    pub transaction_id: String,
    pub converted_amount: f64,
    pub exchange_rate: f64,
    pub fee: f64,
}

// --- WITHDRAW DTOS ---
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct WithdrawRequest {
    pub amount: f64,
    pub currency: String,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WithdrawResponse {
    pub transaction_id: String,
}

// --- TRANSFER DTOS ---
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TransferRequest {
    pub amount: f64,
    pub currency: String,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct WalletTransferRequest {
    pub recipient_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}

// --- TOP-UP DTOS ---
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TopUpRequest {
    #[validate(range(min = 1.0, max = 10000.0))]
    pub amount: f64,
    pub provider: String,
    pub currency: String,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TopUpResponse {
    pub session_url: Option<String>,
    pub payment_id: Option<String>,
    pub transaction_id: String,
    pub amount: f64,
}

// --- TRANSACTION DTOS ---
#[derive(Serialize, ToSchema, Debug)]
pub struct TransactionResponse {
    pub id: String,
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub created_at: String,
    pub status: String,
    pub notes: Option<String>,
}

// --- REGISTRATION DTOS ---
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 3))]
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// --- RESPONSE DTOS ---
#[derive(Serialize, ToSchema, Debug)]
pub struct UserDto {
    pub email: String,
    pub username: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserDto,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct RegisterResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_email: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_email: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

pub struct RefreshResult {
    pub user_id: Uuid,
    pub new_refresh_token: String,
}
#[derive(Deserialize)]
pub struct PaystackRecipientResponse {
    pub status: bool,
    pub data: PaystackRecipientData,
}

#[derive(Deserialize)]
pub struct PaystackRecipientData {
    pub recipient_code: String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct PaystackBank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: String,
    pub country: String,
    pub gateway: Option<String>,
    pub pay_with_bank: Option<bool>,
    pub is_active: Option<bool>,
}

