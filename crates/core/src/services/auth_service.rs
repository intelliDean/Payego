use argon2::{password_hash::PasswordHash, Argon2, Params, PasswordVerifier};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use hex;
use password_hash::PasswordHasher;
use payego_primitives::config::security_config::{Claims, SecurityConfig};
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::authentication::{NewRefreshToken, RefreshToken};
use payego_primitives::models::dtos::dtos::{RefreshResult, RegisterRequest, RegisterResponse};
use payego_primitives::models::{
    app_state::app_state::AppState,
    dtos::dtos::{LoginRequest, LoginResponse},
    user::User,
};
use payego_primitives::schema::refresh_tokens::dsl::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use tracing::{error, warn};
use tracing::log::info;
use utoipa::ToSchema;
use payego_primitives::models::enum_types::CurrencyCode;
use payego_primitives::models::user::NewUser;
use payego_primitives::schema::{blacklisted_tokens, users, wallets};

#[derive(Insertable)]
#[diesel(table_name = blacklisted_tokens)]
struct NewBlacklistedToken<'a> {
    jti: &'a str,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletSummaryDto {
    pub currency: CurrencyCode,
    pub balance: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserResponse {
    pub email: String,
    pub wallets: Vec<WalletSummaryDto>,
}

pub struct AuthService;

impl AuthService {
    //======= LOGIN =============
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

        let argon2 = Self::create_argon2()?;

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
        super::auth_service::AuthService::generate_refresh_token(conn, user_uuid).map_err(|_| {
            error!("auth.login: refresh token creation failed");
            ApiError::Internal("Authentication service unavailable".into())
        })
    }

    fn dummy_hash() -> &'static str {
        "$argon2id$v=19$m=65536,t=3,p=1$\
         c29tZXNhbHQ$\
         c29tZWZha2VoYXNo"
    }

    //====== LOGOUT ===========

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

    //============= REGISTER =============
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

        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(&mut conn)
            .map_err(|e| {
                if Self::is_unique_violation(&e) {
                    info!("auth.register: duplicate email");
                    ApiError::Auth(AuthError::InternalError("Email already exist".to_string()))
                } else {
                    error!("auth.register: failed to insert user");
                    ApiError::Internal("Registration failed".into())
                }
            })?;

        let token = SecurityConfig::create_token(state, &user.id.to_string())
            .map_err(|_| {
                error!("auth.register: jwt generation failed");
                ApiError::Internal("Authentication service error".into())
            })?;

        let refresh_token = Self::generate_refresh_token(&mut conn, user.id)
            .map_err(|_| {
                error!("auth.register: refresh token generation failed");
                ApiError::Internal("Authentication service error".into())
            })?;

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

    fn is_unique_violation(err: &diesel::result::Error) -> bool {
        matches!(
            err,
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            )
        )
    }

    //====================================
    pub fn generate_refresh_token(
        conn: &mut PgConnection,
        user_uuid: Uuid,
    ) -> Result<String, ApiError> {
        use rand::rngs::OsRng;

        let raw_token: String = OsRng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let hashed_token = Self::hash_token(&raw_token);
        let expiry = Utc::now() + Duration::days(7);

        diesel::insert_into(refresh_tokens)
            .values(NewRefreshToken {
                user_id: user_uuid,
                token_hash: &hashed_token,
                expires_at: expiry,
            })
            .execute(conn)
            .map_err(|e: diesel::result::Error| ApiError::from(e))?;

        Ok(raw_token)
    }

    pub fn validate_and_rotate_refresh_token(
        state: &AppState,
        raw_token: &str,
    ) -> Result<RefreshResult, ApiError> {
        let mut conn = state.db.get().map_err(|e| {
            tracing::error!("DB connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;
        
        let hashed_token = Self::hash_token(raw_token);

        let token_record = diesel::update(
            refresh_tokens
                .filter(token_hash.eq(&hashed_token))
                .filter(revoked.eq(false))
                .filter(expires_at.gt(Utc::now())),
        )
        .set(revoked.eq(true))
        .get_result::<RefreshToken>(&mut conn)
        .optional()?;

        if let Some(token_record) = token_record {
            let new_token = Self::generate_refresh_token(&mut conn, token_record.user_id)?;

            Ok(RefreshResult {
                user_id: token_record.user_id,
                new_refresh_token: new_token,
            })
        } else {
            Err(ApiError::Auth(AuthError::InvalidToken(
                "Invalid or expired refresh token".into(),
            )))
        }
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

    //========= USER ============
    pub async fn current_user_summary(
        state: &AppState,
        usr_id: Uuid,
    ) -> Result<CurrentUserResponse, ApiError> {
        let mut conn = state.db.get().map_err(|_| {
            error!("user.summary: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let email = users::table
            .find(usr_id)
            .select(users::email)
            .first::<String>(&mut conn)
            .optional()
            .map_err(|_| {
                error!("user.summary: failed to fetch user email");
                ApiError::Internal("Failed to load user".into())
            })?
            .ok_or_else(|| {
                ApiError::Auth(AuthError::InvalidToken("User does not exist".into()))
            })?;

        let wallets = wallets::table
            .filter(wallets::user_id.eq(usr_id))
            .select((wallets::currency, wallets::balance))
            .load::<(CurrencyCode, i64)>(&mut conn)
            .map_err(|_| {
                error!("user.summary: failed to load wallets");
                ApiError::Internal("Failed to load wallets".into())
            })?
            .into_iter()
            .map(|(currency, balance)| WalletSummaryDto { currency, balance })
            .collect();

        Ok(CurrentUserResponse { email, wallets })
    }
}











