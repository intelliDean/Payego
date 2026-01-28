use crate::services::auth_service::token::TokenService;
use crate::services::audit_service::AuditService;
use argon2::{Argon2, Params};

use crate::repositories::user_repository::UserRepository;
use password_hash::PasswordHasher;
pub use payego_primitives::{
    error::{ApiError, AuthError},
    models::{
        dtos::auth_dto::{RegisterRequest, RegisterResponse},
        user::NewUser,
        user::User,
    },
    schema::users,
};
pub use crate::app_state::AppState;
pub use crate::security::SecurityConfig;
use secrecy::{ExposeSecret, SecretString};
use tracing::{error, info};

pub struct RegisterService;

impl RegisterService {
    pub async fn register(
        state: &AppState,
        payload: RegisterRequest,
    ) -> Result<RegisterResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("auth.register: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let password = SecretString::new(payload.password.into());

        let password_hash = Self::hash_password(&password)?;

        let new_user = NewUser {
            email: &payload.email,
            password_hash: &password_hash,
            username: payload.username.as_deref(),
        };

        let user = UserRepository::create(&mut conn, new_user)?;

        let token = SecurityConfig::create_token(state, &user.id.to_string()).map_err(|_| {
            error!("auth.register: jwt generation failed");
            ApiError::Internal("Authentication service error".into())
        })?;

        let refresh_token =
            TokenService::generate_refresh_token(&mut conn, user.id).map_err(|_| {
                error!("auth.register: refresh token generation failed");
                ApiError::Internal("Authentication service error".into())
            })?;

        let _ = AuditService::log_event(
            state,
            Some(user.id),
            "auth.register",
            Some("user"),
            Some(&user.id.to_string()),
            serde_json::json!({ "email": user.email }),
            None,
        )
        .await;

        info!(
            user_id = %user.id,
            email = %user.email,
            "User registered successfully"
        );

        Ok(RegisterResponse {
            token,
            refresh_token,
            user_email: user.email,
        })
    }

    fn hash_password(password: &SecretString) -> Result<String, ApiError> {
        let argon2 = Self::create_argon2()?;
        let salt = argon2::password_hash::SaltString::generate(&mut rand_core::OsRng);

        argon2
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| {
                error!("auth.register: password hashing failed");
                ApiError::Internal("Credential processing failed".into())
            })
    }

    pub fn create_argon2() -> Result<Argon2<'static>, ApiError> {
        let params = Params::new(
            65536, // 64 MiB memory
            3,     // iterations
            1,     // parallelism
            None,
        )
        .map_err(|e| {
            error!("Argon2 params error: {}", e);
            ApiError::Internal("Encryption configuration error".to_string())
        })?;
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
        Ok(argon2)
    }
}
