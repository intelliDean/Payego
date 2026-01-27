use crate::repositories::user_repository::UserRepository;
use crate::repositories::wallet_repository::WalletRepository;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::AppState, dtos::auth_dto::CurrentUserResponse,
        dtos::wallet_dto::WalletSummaryDto, enum_types::CurrencyCode,
    },
    schema::{users, wallets},
};
use tracing::log::error;
use uuid::Uuid;

pub struct UserService;

impl UserService {
    pub async fn current_user_summary(
        state: &AppState,
        usr_id: Uuid,
    ) -> Result<CurrentUserResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("user.summary: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let user_data = UserRepository::find_by_id(&mut conn, usr_id)?
            .ok_or_else(|| ApiError::Auth(AuthError::InvalidToken("User does not exist".into())))?;

        let walletz = WalletRepository::find_all_by_user(&mut conn, usr_id)?
            .into_iter()
            .map(|w| WalletSummaryDto {
                currency: w.currency,
                balance: w.balance,
            })
            .collect();

        Ok(CurrentUserResponse {
            id: user_data.id,
            email: user_data.email,
            username: user_data.username,
            wallets: walletz,
            created_at: user_data.created_at,
        })
    }
}
