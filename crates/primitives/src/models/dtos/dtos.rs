use diesel::{Queryable, Selectable};
use reqwest::Client;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use crate::error::ApiError;
use crate::models::enum_types::{CurrencyCode, PaymentState};

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

// #[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
// pub struct WithdrawRequest {
//     pub amount: f64,
//     pub currency: CurrencyCode,
//     pub reference: Uuid,
//     pub idempotency_key: String,
// }
// 
// #[derive(Debug, Serialize, ToSchema)]
// pub struct WithdrawResponse {
//     pub transaction_id: Uuid,
// }

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
    pub currency: CurrencyCode,
    pub description: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}




#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct TopUpRequest {
    #[validate(range(min = 1.0, max = 10000.0))]
    pub amount: f64,
    pub provider: PaymentProvider,
    pub currency: String,
    pub reference: Uuid,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PaymentProvider {
    Stripe,
    Paypal,
}


// #[derive(Debug, Clone)]
// pub enum PaymentProvider {
//     Stripe,
//     Paypal,
// }
//
// impl PaymentProvider {
//     pub fn from_str(s: &str) -> Result<Self, ApiError> {
//         match s {
//             "stripe" => Ok(Self::Stripe),
//             "paypal" => Ok(Self::Paypal),
//             _ => Err(ApiError::Payment(format!("Unsupported provider: {s}"))),
//         }
//     }
//
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             Self::Stripe => "stripe",
//             Self::Paypal => "paypal",
//         }
//     }
// }




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
    pub currency: CurrencyCode,
    pub country: String,
    pub gateway: Option<String>,
    pub pay_with_bank: Option<bool>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct PayPalCapture {
    pub status: String,
    pub amount: PayPalAmount,
}

#[derive(Debug, Deserialize)]
pub struct PayPalPayments {
    pub captures: Vec<PayPalCapture>,
}

#[derive(Debug, Deserialize)]
pub struct PayPalPurchaseUnit {
    pub payments: PayPalPayments,
}

#[derive(Deserialize, ToSchema)]
pub struct CaptureRequest {
    pub order_id: String,
    pub transaction_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PayPalCaptureResponse {
    pub status: String,
    pub purchase_units: Vec<PurchaseUnit>,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseUnit {
    pub payments: Payments,
}

#[derive(Debug, Deserialize)]
pub struct Payments {
    pub captures: Vec<Capture>,
}

#[derive(Debug, Deserialize)]
pub struct Capture {
    pub amount: PayPalAmount,
}

#[derive(Debug, Deserialize)]
pub struct PayPalAmount {
    pub currency_code: String,
}

#[derive(Serialize, ToSchema)]
pub struct CaptureResponse {
    pub status: PaymentState,
    pub transaction_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaystackWebhook {
    pub event: String,
    pub data: PaystackData,
}
#[derive(Debug, Deserialize, ToSchema)]
pub struct PaystackData {
    pub reference: String,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionStatus {
    Pending,
    Completed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResolvedAccount {
    pub account_name: String,
    pub bank_code: String,
    pub account_number: String,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(rename_all = "lowercase")]
// pub enum PaymentProvider {
//     Stripe,
//     Paypal,
// }
//
// impl PaymentProvider {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             Self::Stripe => "stripe",
//             Self::Paypal => "paypal",
//         }
//     }
// }



#[derive(Debug, Deserialize)]
pub struct PayPalTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct PayPalOrderResponse {
    pub status: String,
}


impl PaymentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::RequiresAction => "requires_action",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::RequiresAction)
                | (Self::Pending, Self::Completed)
                | (Self::RequiresAction, Self::Completed)
                | (Self::Pending, Self::Failed)
                | (Self::RequiresAction, Self::Failed)
        )
    }

    pub fn from_str(s: &str) -> Result<Self, ApiError> {
        match s {
            "pending" => Ok(Self::Pending),
            "requires_action" => Ok(Self::RequiresAction),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(ApiError::Internal("Invalid payment state".into())),
        }
    }

    // pub fn transition_transaction_state(
    //     conn: &mut PgConnection,
    //     transaction_id: Uuid,
    //     next: PaymentState,
    // ) -> Result<Transaction, ApiError> {
    //     use payego_primitives::schema::transactions::dsl::*;
    // 
    //     let current: Transaction = transactions
    //         .filter(reference.eq(transaction_id))
    //         .first(conn)
    //         .map_err(ApiError::from)?;
    // 
    //     let current_state =
    //         PaymentState::from_str(&current.payment_state)?;
    // 
    //     if !current_state.can_transition_to(next) {
    //         return Err(ApiError::Payment(format!(
    //             "Illegal state transition: {:?} → {:?}",
    //             current_state, next
    //         )));
    //     }
    // 
    //     diesel::update(transactions.filter(reference.eq(transaction_id)))
    //         .set(payment_state.eq(next.as_str()))
    //         .returning(Transaction::as_select())
    //         .get_result(conn)
    //         .map_err(ApiError::from)
    // }

}
