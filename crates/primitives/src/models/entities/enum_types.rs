use crate::error::ApiError;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
// Kept for mixed usage if needed, but aiming for separation

//cargo run -- src/.../entities/your_file.rs

use strum::{Display, EnumString};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum, Display, EnumString, ToSchema,
)]
#[ExistingTypePath = "crate::schema::sql_types::CurrencyCode"]
#[strum(serialize_all = "UPPERCASE")]
pub enum CurrencyCode {
    USD,
    NGN,
    GBP,
    EUR,
    CAD,
    AUD,
    CHF,
    JPY,
    CNY,
    SEK,
    NZD,
    MXN,
    SGD,
    HKD,
    NOK,
    KRW,
    TRY,
    INR,
    BRL,
    ZAR,
}

impl CurrencyCode {
    pub fn parse(input: &str) -> Result<Self, ApiError> {
        let normalized = input.trim().to_uppercase();

        CurrencyCode::from_str(&normalized)
            .map_err(|_| ApiError::Internal(format!("Unsupported currency: {}", input)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TransactionIntent"]
pub enum TransactionIntent {
    TopUp,
    Payout,
    Transfer,
    Conversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentState"]
pub enum PaymentState {
    Pending,
    RequiresAction,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum, ToSchema)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentProvider"]
pub enum PaymentProvider {
    Stripe,
    Paypal,
    Paystack,
    Internal,
}
