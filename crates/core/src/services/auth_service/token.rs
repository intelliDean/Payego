use crate::repositories::token_repository::TokenRepository;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use hex;
pub use payego_primitives::{
    error::{ApiError, AuthError},
    models::{
        authentication::{NewRefreshToken, RefreshToken},
        dtos::auth_dto::{LoginResponse, RefreshRequest, RefreshResponse, RefreshResult},
    },
    schema::refresh_tokens::dsl::*,
};
pub use crate::app_state::AppState;
pub use crate::security::SecurityConfig;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct TokenService;

impl TokenService {
    pub fn generate_refresh_token(
        conn: &mut PgConnection,
        user_uuid: Uuid,
    ) -> Result<String, ApiError> {
        use rand::rngs::OsRng;

        let raw_token: String = OsRng.sample_iter(&Alphanumeric).take(64).collect();

        let hashed_token = Self::hash_token(&raw_token);
        let expiry = Utc::now() + Duration::days(7);

        TokenRepository::create_refresh_token(
            conn,
            NewRefreshToken {
                user_id: user_uuid,
                token_hash: &hashed_token,
                expires_at: expiry,
            },
        )?;

        Ok(raw_token)
    }

    pub fn validate_and_rotate_refresh_token(
        state: &AppState,
        raw_token: &str,
    ) -> Result<RefreshResult, ApiError> {
        let mut conn = state.db.get().map_err(|e| {
            error!("token.refresh: failed to acquire db connection");
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let hashed_token = Self::hash_token(raw_token);

        let token_record = TokenRepository::rotate_refresh_token(&mut conn, &hashed_token)?;

        if let Some(token_record) = token_record {
            let new_token = Self::generate_refresh_token(&mut conn, token_record.user_id)?;

            info!(
                user_id = %token_record.user_id,
                "Refresh token rotated successfully"
            );

            Ok(RefreshResult {
                user_id: token_record.user_id,
                new_refresh_token: new_token,
            })
        } else {
            warn!("token.refresh: invalid or expired refresh token");
            Err(ApiError::Auth(AuthError::InvalidToken(
                "Invalid or expired refresh token".into(),
            )))
        }
    }

    pub fn revoke_all_refresh_tokens(
        conn: &mut PgConnection,
        user_uuid: Uuid,
    ) -> Result<(), ApiError> {
        TokenRepository::revoke_all_user_tokens(conn, user_uuid)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(())
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}
