pub use crate::app_state::AppState;
use crate::repositories::user_repository::UserRepository;
pub use crate::security::SecurityConfig;
use crate::services::audit_service::AuditService;
use crate::services::auth_service::register::RegisterService;
use argon2::{password_hash::PasswordHash, PasswordVerifier};
use diesel::prelude::*;
pub use payego_primitives::{
    error::{ApiError, AuthError},
    models::{
        dtos::auth_dto::{LoginRequest, LoginResponse},
        user::User,
    },
};
use tracing::{error, info, warn};

pub struct LoginService;

impl LoginService {
    pub async fn login(state: &AppState, payload: LoginRequest) -> Result<LoginResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("auth.login: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let user_opt = UserRepository::find_by_email(&mut conn, &payload.email)?;
        let user = user_opt.ok_or_else(|| {
            warn!("auth.login: user not found for email {}", payload.email);
            ApiError::Auth(AuthError::InvalidCredentials)
        })?;

        if let Err(e) = Self::verify_password(&payload.password, &user) {
            let _ = AuditService::log_event(
                state,
                Some(user.id),
                "auth.login.failure",
                None,
                None,
                serde_json::json!({ "reason": "invalid_password" }),
                None,
            )
            .await;
            return Err(e);
        }

        let token = SecurityConfig::create_token(state, &user.id.to_string()).map_err(|_| {
            error!("auth.login: jwt creation failed");
            ApiError::Internal("Authentication service unavailable".into())
        })?;

        let refresh_token = Self::create_refresh_token(&mut conn, user.id)?;

        let _ = AuditService::log_event(
            state,
            Some(user.id),
            "auth.login.success",
            None,
            None,
            serde_json::json!({}),
            None,
        )
        .await;

        info!(
            user_id = %user.id,
            "User logged in successfully"
        );

        Ok(LoginResponse {
            token,
            refresh_token,
            user_email: Some(user.email),
        })
    }

    fn verify_password(password: &str, user: &User) -> Result<(), ApiError> {
        let hash = &user.password_hash;

        let parsed = PasswordHash::new(hash).map_err(|_| {
            error!("auth.login: invalid password hash");
            ApiError::Internal("Authentication failure".into())
        })?;

        let argon2 = RegisterService::create_argon2()?;

        if argon2
            .verify_password(password.as_bytes(), &parsed)
            .is_err()
        {
            warn!("auth.login: invalid credentials");
            return Err(ApiError::Auth(AuthError::InvalidCredentials));
        }

        Ok(())
    }

    fn create_refresh_token(
        conn: &mut PgConnection,
        user_uuid: uuid::Uuid,
    ) -> Result<String, ApiError> {
        super::token::TokenService::generate_refresh_token(conn, user_uuid).map_err(|_| {
            error!("auth.login: refresh token creation failed");
            ApiError::Internal("Authentication service unavailable".into())
        })
    }

    // fn dummy_hash() -> &'static str {
    //     "$argon2id$v=19$m=65536,t=3,p=1$\
    //      c29tZXNhbHQ$\
    //      c29tZWZha2VoYXNo"
    // }
}
