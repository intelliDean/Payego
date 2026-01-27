use crate::models::withdrawal_dto::WalletSummaryDto;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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
}
