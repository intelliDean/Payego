use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use uuid::Uuid;
use crate::utility::validate_password;

// Request DTOs

#[derive(Deserialize, ToSchema, Validate, Debug)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8), custom(function = "validate_password"))]
    pub password: String,
    #[validate(length(
        min = 3,
        max = 100,
        message = "Username must be between 3 and 100 characters"
    ))]
    pub username: Option<String>,
}

#[derive(Deserialize, ToSchema, Validate, Debug)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length( min = 8), custom(function = "validate_password"))]
    pub password: String,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct PayoutRequest {
    pub amount: f64, // Amount in NGN
    pub currency: String, // Currency to deduct from (e.g., "USD")
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub reference: Uuid,
    pub idempotency_key: String,
}

// Response DTOs

// Safe User DTO (Excludes password_hash)
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
    pub user_email: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ErrorResponse {
    pub error: String,
}
