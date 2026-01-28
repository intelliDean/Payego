pub use crate::app_state::AppState;
pub use crate::security::Claims;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
pub use payego_primitives::{
    error::ApiError,
    models::{dtos::auth_dto::LogoutResponse, entities::authentication::NewBlacklistedToken},
};
use tracing::{error, info};

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
            .values(NewBlacklistedToken {
                jti: jti_val,
                expires_at: expiration,
            })
            .on_conflict(jti)
            .do_nothing()
            .execute(conn)
            .map_err(|_| {
                error!("auth.logout: failed to persist token blacklist");
                ApiError::Internal("Logout failed".into())
            })?;

        info!("auth.logout: token invalidated");

        Ok(())
    }
}
