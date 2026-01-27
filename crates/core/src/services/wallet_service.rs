use tracing::{error, warn};
use uuid::Uuid;
use crate::repositories::wallet_repository::WalletRepository;

pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{
        app_state::AppState,
        wallet::Wallet,
        wallet_dto::{WalletDto, WalletsResponse},
    },
    schema::wallets,
};

pub struct WalletService;

impl WalletService {
    pub async fn get_user_wallets(
        state: &AppState,
        claims: &Claims,
    ) -> Result<WalletsResponse, ApiError> {
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            warn!("wallets.list: invalid subject in token");
            ApiError::Auth(AuthError::InvalidToken("Invalid token".into()))
        })?;

        let mut conn = state.db.get().map_err(|_| {
            error!("wallets.list: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let wallets = WalletRepository::find_all_by_user(&mut conn, user_id)
            .map_err(|_| {
                error!("wallets.list: query failed");
                ApiError::Internal("Failed to fetch wallets".into())
            })?;

        Ok(WalletsResponse {
            wallets: wallets.into_iter().map(WalletDto::from).collect(),
        })
    }
}
