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
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

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

        let app_url =
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
        let verification_url = format!("{}/verify-email?token={}", app_url, token);

        let subject = "Verify your email - Payego";
        let body = format!(
            r#"
            <div style="font-family: sans-serif; max-width: 600px; margin: auto; padding: 20px; border: 1px solid #eee; border-radius: 10px;">
                <h2 style="color: #333;">Welcome to Payego!</h2>
                <p>Please verify your email address to get started managing your finances.</p>
                <div style="margin: 30px 0;">
                    <a href="{0}" style="background-color: #7c3aed; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; font-weight: bold;">Verify Email Address</a>
                </div>
                <p style="color: #666; font-size: 14px;">If the button doesn't work, copy and paste this link into your browser:</p>
                <p style="color: #666; font-size: 14px; word-break: break-all;">{0}</p>
                <hr style="border: 0; border-top: 1px solid #eee; margin: 30px 0;">
                <p style="color: #999; font-size: 12px;">This link will expire in 24 hours.</p>
            </div>
            "#,
            verification_url
        );

        state.email.send_email(email, subject, &body).await?;

        Ok(())
    }

    pub async fn verify_email(state: &AppState, token: &str) -> Result<(), ApiError> {
        let token_hash = Self::hash_token(token);

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        VerificationRepository::consume_token(&mut conn, &token_hash)?;

        Ok(())
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}
