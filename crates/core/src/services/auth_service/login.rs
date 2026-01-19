use crate::services::auth_service::register::RegisterService;
use argon2::{password_hash::PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use tracing::{error, warn};
pub use payego_primitives::{
    config::security_config::SecurityConfig,
    error::{ApiError, AuthError},
    models::{
        app_state::app_state::AppState,
        dtos::login_dto::{LoginRequest, LoginResponse},
        user::User,
    },
};

pub struct LoginService;

impl LoginService {
    pub async fn login(state: &AppState, payload: LoginRequest) -> Result<LoginResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("auth.login: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let user = Self::find_user_by_email(&mut conn, &payload.email)?;
        Self::verify_password(&payload.password, user.as_ref())?;

        let user = user.ok_or(ApiError::Auth(AuthError::InvalidCredentials))?;

        let token = SecurityConfig::create_token(state, &user.id.to_string()).map_err(|_| {
            error!("auth.login: jwt creation failed");
            ApiError::Internal("Authentication service unavailable".into())
        })?;

        let refresh_token = Self::create_refresh_token(&mut conn, user.id)?;

        Ok(LoginResponse {
            token,
            refresh_token,
            user_email: Some(user.email),
        })
    }

    fn find_user_by_email(
        conn: &mut PgConnection,
        email_addr: &str,
    ) -> Result<Option<User>, ApiError> {
        use payego_primitives::schema::users::dsl::*;

        users
            .filter(email.eq(email_addr))
            .first::<User>(conn)
            .optional()
            .map_err(|_| {
                error!("auth.login: db query failed");
                ApiError::Internal("Authentication failure".into())
            })
    }

    fn verify_password(password: &str, user: Option<&User>) -> Result<(), ApiError> {
        // Always verify *something* to prevent timing attacks
        let hash = user
            .map(|u| u.password_hash.as_str())
            .unwrap_or(Self::dummy_hash());

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

    fn dummy_hash() -> &'static str {
        "$argon2id$v=19$m=65536,t=3,p=1$\
         c29tZXNhbHQ$\
         c29tZWZha2VoYXNo"
    }
}
