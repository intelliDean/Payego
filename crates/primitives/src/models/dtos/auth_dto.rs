use crate::models::dtos::wallet_dto::WalletSummaryDto;
use crate::utility::validate_password;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,

    #[schema(example = "P@ssw0rd123!", format = "password")]
    pub password: String,
}

impl LoginRequest {
    pub fn normalize(mut self) -> Self {
        self.email = self.email.trim().to_lowercase();
        self
    }
}

#[derive(Serialize, ToSchema, Debug)]
pub struct LoginResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    pub refresh_token: String,
    #[schema(example = "user@example.com")]
    pub user_email: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct RefreshResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "user@example.com")]
    #[validate(email)]
    pub email: String,

    #[schema(example = "P@ssw0rd123!", format = "password")]
    #[validate(custom(function = "validate_password"))]
    pub password: String,

    #[validate(length(min = 3))]
    pub username: Option<String>,
}

impl RegisterRequest {
    pub fn normalize(mut self) -> Self {
        self.email = self.email.trim().to_lowercase();

        // Only set username to email if it wasn't provided or is empty
        if self.username.is_none()
            || self
                .username
                .as_deref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
        {
            self.username = Some(self.email.split('@').next().unwrap().to_string());
        } else {
            self.username = self.username.map(|u| u.trim().to_lowercase());
        }
        self
    }
}

#[derive(Serialize, ToSchema, Debug)]
pub struct RegisterResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_email: String,
}

// --- Token / Session DTOs ---

#[derive(Deserialize, ToSchema, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 64))]
    pub refresh_token: String,
}

pub struct RefreshResult {
    pub user_id: Uuid,
    pub new_refresh_token: String,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({"message": "Successfully logged out", "status": "success"}))]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub wallets: Vec<WalletSummaryDto>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

// --- Health ---

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub message: String,
}
