use argon2::{Params, PasswordVerifier};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::models::{
    app_state::app_state::AppState,
    entities::authentication::NewBlacklistedToken
};
use tracing::log::{error, info};
use payego_primitives::error::ApiError;



pub struct LogoutService;

impl LogoutService {
    pub async fn logout(state: &AppState, claims: Claims) -> Result<(), ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("auth.logout: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let expiration = DateTime::<Utc>::from_timestamp(claims.exp, 0).ok_or_else(|| {
            error!("auth.logout: invalid exp in claims");
            ApiError::Internal("Invalid token".into())
        })?;

        Self::blacklist_token(&mut conn, &claims.jti, expiration)?;

        Ok(())
    }

    fn blacklist_token(
        conn: &mut PgConnection,
        jti_val: &str,
        expiration: DateTime<Utc>,
    ) -> Result<(), ApiError> {
        use payego_primitives::schema::blacklisted_tokens::dsl::*;

        diesel::insert_into(blacklisted_tokens)
            .values(NewBlacklistedToken { jti: jti_val, expires_at: expiration })
            .on_conflict(jti)
            .do_nothing()
            .execute(conn)
            .map_err(|_| {
                error!("auth.logout: failed to persist token blacklist");
                ApiError::Internal("Logout failed".into())
            })?;

        // Idempotent by design — no branching, no noise
        info!("auth.logout: token invalidated");

        Ok(())
    }
}