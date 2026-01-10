use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::{NewRefreshToken, RefreshToken};
use payego_primitives::schema::refresh_tokens::dsl::*;
use chrono::{Duration, Utc};
use hex;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct AuthService;

impl AuthService {
    pub fn generate_refresh_token(
        conn: &mut PgConnection,
        user_uuid: Uuid,
    ) -> Result<String, ApiError> {
        let raw_token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let hashed_token = Self::hash_token(&raw_token);
        let expiry = Utc::now() + Duration::days(7);

        diesel::insert_into(refresh_tokens)
            .values(NewRefreshToken {
                user_id: user_uuid,
                token_hash: hashed_token,
                expires_at: expiry,
            })
            .execute(conn)
            .map_err(|e: diesel::result::Error| ApiError::from(e))?;

        Ok(raw_token)
    }

    pub fn validate_and_rotate_refresh_token(
        conn: &mut PgConnection,
        user_uuid: Uuid,
        raw_token: &str,
    ) -> Result<String, ApiError> {
        let hashed_token = Self::hash_token(raw_token);

        let token_record = refresh_tokens
            .filter(user_id.eq(user_uuid))
            .filter(token_hash.eq(&hashed_token))
            .filter(revoked.eq(false))
            .filter(expires_at.gt(Utc::now()))
            .first::<RefreshToken>(conn)
            .optional()
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::Auth("Invalid or expired refresh token".into()))?;

        diesel::update(refresh_tokens.find(token_record.id))
            .set(revoked.eq(true))
            .execute(conn)
            .map_err(|e: diesel::result::Error| ApiError::from(e))?;

        Self::generate_refresh_token(conn, user_uuid)
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
