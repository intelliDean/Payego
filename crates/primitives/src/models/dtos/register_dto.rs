use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 3))]
    pub username: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct RegisterResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_email: String,
}
