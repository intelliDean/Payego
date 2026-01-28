use crate::repositories::user_repository::UserRepository;
use diesel::prelude::*;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::entities::verification_token::{
    NewVerificationToken, VerificationToken,
};
use payego_primitives::schema::verification_tokens;
use uuid::Uuid;

pub struct VerificationRepository;

impl VerificationRepository {
    pub fn create(
        conn: &mut PgConnection,
        new_token: NewVerificationToken,
    ) -> Result<VerificationToken, ApiError> {
        diesel::insert_into(verification_tokens::table)
            .values(&new_token)
            .get_result(conn)
            .map_err(ApiError::Database)
    }

    pub fn find_by_token(
        conn: &mut PgConnection,
        token_hash: &str,
    ) -> Result<Option<VerificationToken>, ApiError> {
        verification_tokens::table
            .filter(verification_tokens::token_hash.eq(token_hash))
            .first::<VerificationToken>(conn)
            .optional()
            .map_err(ApiError::Database)
    }

    pub fn delete_for_user(conn: &mut PgConnection, user_id: Uuid) -> Result<(), ApiError> {
        diesel::delete(verification_tokens::table.filter(verification_tokens::user_id.eq(user_id)))
            .execute(conn)
            .map(|_| ())
            .map_err(ApiError::Database)
    }

    pub fn consume_token(
        conn: &mut PgConnection,
        token_hash: &str,
    ) -> Result<VerificationToken, ApiError> {
        let token = Self::find_by_token(conn, token_hash)?.ok_or_else(|| {
            ApiError::Auth(AuthError::VerificationError(
                "Invalid or expired verification token".into(),
            ))
        })?;

        if token.expires_at < chrono::Utc::now().naive_utc() {
            return Err(ApiError::Auth(AuthError::VerificationError(
                "Verification token has expired".into(),
            )));
        }

        // Verify user and delete token
        UserRepository::mark_email_verified(conn, token.user_id)?;
        Self::delete_for_user(conn, token.user_id)?;

        Ok(token)
    }
}
