use argon2::{password_hash::PasswordHash, Argon2, Params, PasswordVerifier};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use hex;
use password_hash::PasswordHasher;
use payego_primitives::config::security_config::{Claims, SecurityConfig};
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::authentication::{NewRefreshToken, RefreshToken};
use payego_primitives::models::dtos::token_dto::{RefreshResult,};
use payego_primitives::models::dtos::register_dto::{ RegisterRequest, RegisterResponse,};
use payego_primitives::models::enum_types::CurrencyCode;
use payego_primitives::models::{
    app_state::app_state::AppState,
    dtos::login_dto::{LoginRequest, LoginResponse},
    user::User,
};
use payego_primitives::schema::refresh_tokens::dsl::*;
use payego_primitives::schema::{blacklisted_tokens, users, wallets};
use rand::distributions::Alphanumeric;
use rand::Rng;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tracing::log::info;
use tracing::{error, warn};
use utoipa::ToSchema;
use uuid::Uuid;


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

pub struct UserService;

impl UserService {
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
            .ok_or_else(|| ApiError::Auth(AuthError::InvalidToken("User does not exist".into())))?;

        let walletz = wallets::table
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

        Ok(CurrentUserResponse {
            email,
            wallets: walletz,
        })
    }
}
