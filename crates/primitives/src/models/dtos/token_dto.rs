use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::models::user::User;
use crate::models::withdrawal_dto::WalletSummaryDto;

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
