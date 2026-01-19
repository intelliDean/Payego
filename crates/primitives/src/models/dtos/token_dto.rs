use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

pub struct RefreshResult {
    pub user_id: Uuid,
    pub new_refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}
