use crate::app_state::AppState;
use crate::repositories::verification_repository::VerificationRepository;
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::verification_token::NewVerificationToken;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct VerificationService;

impl VerificationService {
    pub async fn send_verification_email(
        state: &AppState,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), ApiError> {
        let token = Uuid::new_v4().to_string();
        let token_hash = Self::hash_token(&token);

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::Database(e.to_string()))?;

        // 24 hour expiry
        let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::hours(24);

        VerificationRepository::delete_for_user(&mut conn, user_id)?;
        VerificationRepository::create(
            &mut conn,
            NewVerificationToken {
                user_id,
                token_hash,
                expires_at,
            },
        )?;

        let subject = "Verify your email - Payego";
        let body = format!(
            "Please verify your email by clicking here: /verify-email?token={}",
            token
        );

        state.email.send_email(email, subject, &body).await?;

        Ok(())
    }

    pub async fn verify_email(state: &AppState, token: &str) -> Result<(), ApiError> {
        let token_hash = Self::hash_token(token);
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::Database(e.to_string()))?;

        VerificationRepository::consume_token(&mut conn, &token_hash)?;

        Ok(())
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}
