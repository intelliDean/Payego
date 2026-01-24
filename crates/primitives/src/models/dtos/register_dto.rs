use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use crate::utility::validate_password;

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
        // self.password = self.password.trim().to_string();
        
        // Only set username to email if it wasn't provided or is empty
        if self.username.is_none() || self.username.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
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
