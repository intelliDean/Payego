use diesel::prelude::*;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::app_state::AppState, enum_types::CurrencyCode, token_dto::CurrentUserResponse,
        withdrawal_dto::WalletSummaryDto,
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

        let email = users::table
            .find(usr_id)
            .select(users::email)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|_| {
                error!("user.summary: failed to fetch user email");
                ApiError::Internal("Failed to load user".into())
            })?
            .ok_or_else(|| ApiError::Auth(AuthError::InvalidToken("User does not exist".into())))?;

        let walletz = wallets::table
            .filter(wallets::user_id.eq(usr_id))
            .select((wallets::currency, wallets::balance))
            .load::<(CurrencyCode, i64)>(&mut conn)
            .map_err(|_| {
                error!("user.summary: failed to load wallets");
                ApiError::Internal("Failed to load wallets".into())
            })?
            .into_iter()
            .map(|(currency, balance)| WalletSummaryDto { currency, balance })
            .collect();

        Ok(CurrentUserResponse {
            email,
            wallets: walletz,
        })
    }
}
