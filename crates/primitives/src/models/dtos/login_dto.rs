use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_email: Option<String>,
}
