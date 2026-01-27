use crate::models::enum_types::{CurrencyCode, PaymentProvider};
use crate::models::wallet::Wallet;
use chrono::{DateTime, Utc};
use diesel::Queryable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// --- Wallet & Balance DTOs ---

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletDto {
    pub id: Uuid,
    pub currency: CurrencyCode,
    pub balance: i64, // cents
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletsResponse {
    pub wallets: Vec<WalletDto>,
}

impl From<Wallet> for WalletDto {
    fn from(wallet: Wallet) -> Self {
        Self {
            id: wallet.id,
            currency: wallet.currency,
            balance: wallet.balance,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletSummaryDto {
    pub currency: CurrencyCode,
    pub balance: i64,
}

// --- Transfer DTOs ---

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TransferRequest {
    #[validate(range(min = 1.0, max = 10000.0))]
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
    pub recipient: Uuid,
    #[validate(range(min = 1.0, max = 10000.0))]
    pub amount: f64,
    pub currency: CurrencyCode,
    pub description: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Deserialize)]
pub struct ResolveUserRequest {
    pub identifier: String,
}

#[derive(Serialize, Queryable, ToSchema)]
pub struct ResolvedUser {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct TransferResponse {
    pub transaction_id: Uuid,
}

// --- Withdrawal DTOs ---

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct WithdrawRequest {
    #[validate(range(min = 0.01))]
    pub amount: f64,
    pub currency: CurrencyCode,
    pub reference: Uuid,
    #[validate(length(min = 10, max = 128))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WithdrawResponse {
    pub transaction_id: Uuid,
}

// --- Top Up DTOs ---

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TopUpRequest {
    #[validate(range(min = 1.0, max = 10_000.0))]
    pub amount: f64,
    pub provider: PaymentProvider,
    #[schema(example = "NGN")]
    pub currency: CurrencyCode,
    #[validate(length(min = 8, max = 128))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopUpResponse {
    pub session_url: Option<String>,
    pub payment_id: Option<String>,
    pub transaction_id: String,
    pub amount: f64,
}

// --- Conversion DTOs ---

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ConvertRequest {
    #[validate(range(min = 1))]
    pub amount_cents: i64,
    pub from_currency: CurrencyCode,
    pub to_currency: CurrencyCode,
    #[validate(length(min = 8, max = 128))]
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConvertResponse {
    pub transaction_id: String,
    pub converted_amount: f64,
    pub exchange_rate: f64,
    pub fee: f64,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRateResponse {
    pub rates: HashMap<String, f64>,
    pub error: Option<String>,
}

pub struct ConvertQuoteRequest {
    pub from_currency: CurrencyCode,
    pub to_currency: CurrencyCode,
    pub amount_cents: i64,
}

pub struct ConvertQuoteResponse {
    pub quote_id: Uuid,
    pub exchange_rate: f64,
    pub fee: f64,
    pub net_amount: f64,
    pub expires_at: DateTime<Utc>,
}

pub struct ConfirmConvertRequest {
    pub quote_id: Uuid,
    pub idempotency_key: String,
}
