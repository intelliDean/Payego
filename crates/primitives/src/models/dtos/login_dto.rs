use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
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
    pub refresh_token: String
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub message: String,
}