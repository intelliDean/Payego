use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use crate::models::user::User;
use crate::models::withdrawal_dto::WalletSummaryDto;


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
pub struct LogoutResponse {
    pub message: String,
}


#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserResponse {
    pub email: String,
    pub wallets: Vec<WalletSummaryDto>,
}
