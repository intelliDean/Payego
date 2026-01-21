use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "P@ssw0rd123!", format = "password")]
    pub password: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct LoginResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    pub refresh_token: String,
    #[schema(example = "user@example.com")]
    pub user_email: Option<String>,
}
