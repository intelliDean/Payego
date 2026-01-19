use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use crate::models::bank::{Bank, BankAccount};


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
pub struct PaystackAccountData {
    pub account_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResolvedAccount {
    pub account_name: String,
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackBank {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "active", default)]
    pub is_active: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ResolveAccountRequest {
    pub bank_code: String,
    pub account_number: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResolveAccountResponse {
    pub account_name: String,
}


#[derive(Debug, Deserialize)]
pub struct PaystackResolveResponse {
   pub  status: bool,
    pub message: String,
    pub data: Option<PaystackAccountData>,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BankDto {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub currency: String,
    pub is_active: bool,
}

impl From<Bank> for BankDto {
    fn from(bank: Bank) -> Self {
        Self {
            id: bank.id,
            name: bank.name,
            code: bank.code,
            currency: bank.currency.to_string(),
            is_active: bank.is_active,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BankListResponse {
    pub banks: Vec<BankDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankAccountResponse {
    pub id: Uuid,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub bank_name: Option<String>,
    pub is_verified: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BankAccountsResponse {
    pub bank_accounts: Vec<BankAccountResponse>,
}

impl From<BankAccount> for BankAccountResponse {
    fn from(account: BankAccount) -> Self {
        Self {
            id: account.id,
            bank_code: account.bank_code,
            account_number: account.account_number,
            account_name: account.account_name,
            bank_name: account.bank_name,
            is_verified: account.is_verified,
        }
    }
}