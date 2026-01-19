use argon2::{password_hash::PasswordHash, Argon2, Params, PasswordVerifier};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use hex;
use password_hash::PasswordHasher;
use payego_primitives::config::security_config::{Claims, SecurityConfig};
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::authentication::{NewRefreshToken, RefreshToken};
use payego_primitives::models::dtos::{token_dto::RefreshResult, {register_dto::{RegisterResponse, RegisterRequest} }};
use payego_primitives::models::{
    app_state::app_state::AppState,
    dtos::login_dto::{LoginRequest, LoginResponse},
    user::User,
};
use payego_primitives::schema::refresh_tokens::dsl::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use tracing::{error, warn};
use tracing::log::info;
use utoipa::ToSchema;
use payego_primitives::models::enum_types::CurrencyCode;
use payego_primitives::models::user::NewUser;
use payego_primitives::schema::{blacklisted_tokens, users, wallets};



pub struct TokenService;

impl TokenService {
    pub fn generate_refresh_token(
        conn: &mut PgConnection,
        user_uuid: Uuid,
    ) -> Result<String, ApiError> {
        use rand::rngs::OsRng;

        let raw_token: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let hashed_token = Self::hash_token(&raw_token);
        let expiry = Utc::now() + Duration::days(7);

        diesel::insert_into(refresh_tokens)
            .values(NewRefreshToken {
                user_id: user_uuid,
                token_hash: &hashed_token,
                expires_at: expiry,
            })
            .execute(conn)
            .map_err(|e: diesel::result::Error| ApiError::from(e))?;

        Ok(raw_token)
    }

    pub fn validate_and_rotate_refresh_token(
        state: &AppState,
        raw_token: &str,
    ) -> Result<RefreshResult, ApiError> {
        let mut conn = state.db.get().map_err(|e| {
            tracing::error!("DB connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let hashed_token = Self::hash_token(raw_token);

        let token_record = diesel::update(
            refresh_tokens
                .filter(token_hash.eq(&hashed_token))
                .filter(revoked.eq(false))
                .filter(expires_at.gt(Utc::now())),
        )
        .set(revoked.eq(true))
        .get_result::<RefreshToken>(&mut conn)
        .optional()?;

        if let Some(token_record) = token_record {
            let new_token = Self::generate_refresh_token(&mut conn, token_record.user_id)?;

            Ok(RefreshResult {
                user_id: token_record.user_id,
                new_refresh_token: new_token,
            })
        } else {
            Err(ApiError::Auth(AuthError::InvalidToken(
                "Invalid or expired refresh token".into(),
            )))
        }
    }

    pub fn revoke_all_refresh_tokens(
        conn: &mut PgConnection,
        user_uuid: Uuid,
    ) -> Result<(), ApiError> {
        diesel::update(refresh_tokens.filter(user_id.eq(user_uuid)))
            .set(revoked.eq(true))
            .execute(conn)
            .map_err(|e: diesel::result::Error| ApiError::from(e))?;
        Ok(())
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

}











