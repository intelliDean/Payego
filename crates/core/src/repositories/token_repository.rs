use chrono::Utc;
use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::authentication::{
    NewBlacklistedToken, NewRefreshToken, RefreshToken,
};
use payego_primitives::schema::{blacklisted_tokens, refresh_tokens};
use uuid::Uuid;

pub struct TokenRepository;

impl TokenRepository {
    pub fn create_refresh_token(
        conn: &mut PgConnection,
        new_token: NewRefreshToken,
    ) -> Result<RefreshToken, ApiError> {
        diesel::insert_into(refresh_tokens::table)
            .values(&new_token)
            .get_result::<RefreshToken>(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_active_refresh_token(
        conn: &mut PgConnection,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, ApiError> {
        refresh_tokens::table
            .filter(refresh_tokens::token_hash.eq(token_hash))
            .filter(refresh_tokens::revoked.eq(false))
            .filter(refresh_tokens::expires_at.gt(Utc::now()))
            .first::<RefreshToken>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn rotate_refresh_token(
        conn: &mut PgConnection,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, ApiError> {
        diesel::update(
            refresh_tokens::table
                .filter(refresh_tokens::token_hash.eq(token_hash))
                .filter(refresh_tokens::revoked.eq(false))
                .filter(refresh_tokens::expires_at.gt(Utc::now())),
        )
        .set(refresh_tokens::revoked.eq(true))
        .get_result::<RefreshToken>(conn)
        .optional()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn revoke_all_user_tokens(conn: &mut PgConnection, user_id: Uuid) -> Result<(), ApiError> {
        diesel::update(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user_id)))
            .set(refresh_tokens::revoked.eq(true))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn blacklist_token(
        conn: &mut PgConnection,
        new_blacklist: NewBlacklistedToken,
    ) -> Result<(), ApiError> {
        diesel::insert_into(blacklisted_tokens::table)
            .values(&new_blacklist)
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn is_token_blacklisted(conn: &mut PgConnection, jti: &str) -> Result<bool, ApiError> {
        let count: i64 = blacklisted_tokens::table
            .filter(blacklisted_tokens::jti.eq(jti))
            .count()
            .get_result(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(count > 0)
    }
}
