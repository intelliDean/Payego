use crate::error::ApiError;
use crate::models::models::AppState;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use http::HeaderMap;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tracing::log::info;
use tracing::{error, warn};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

pub struct JWTSecret {
    pub jwt_secret: String,
}

impl JWTSecret {
    pub fn new() -> Self {
        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment variables");

        if jwt_secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 characters long");
        }

        Self { jwt_secret }
    }
}

pub fn create_token(state: &AppState, user_id: &str) -> Result<String, ApiError> {
    let secret = state.jwt_secret.as_bytes();

    let now = Utc::now();
    let expiration_hours: i64 = env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "2".to_string())
        .parse()
        .map_err(|e| {
            error!("JWT expiration config error: {}", e);
            ApiError::Token(format!("Invalid JWT expiration configuration: {}", e))
        })?;

    let exp = (now + Duration::hours(expiration_hours)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| {
        error!("JWT encoding error: {}", e);
        ApiError::Token(format!("Token creation failed: {}", e))
    })?;

    info!(
        "Generated token for user {} ending in ...{}",
        user_id,
        token.chars().rev().take(8).collect::<String>()
    );
    Ok(token)
}

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidFormat,
    InvalidToken(String),
    BlacklistedToken,
    InternalError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "Authorization header required"),
            AuthError::InvalidFormat => write!(f, "Invalid Authorization format"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::BlacklistedToken => write!(f, "Token has been invalidated"),
            AuthError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl Into<(StatusCode, String)> for AuthError {
    fn into(self) -> (StatusCode, String) {
        match self {
            AuthError::MissingHeader => (
                StatusCode::UNAUTHORIZED,
                "Authorization header required".to_string(),
            ),
            AuthError::InvalidFormat => (
                StatusCode::BAD_REQUEST,
                "Invalid Authorization format".to_string(),
            ),
            AuthError::InvalidToken(msg) => {
                (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", msg))
            }
            AuthError::BlacklistedToken => (
                StatusCode::UNAUTHORIZED,
                "Token has been invalidated".to_string(),
            ),
            AuthError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", msg),
            ),
        }
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidFormat)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)?
        .trim();

    if token.is_empty() {
        return Err(AuthError::InvalidFormat);
    }

    Ok(token.to_string())
}

pub fn verify_token(state: &AppState, token: &str) -> Result<Claims, String> {
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| format!("JWT verification error: {}", e))
}

pub fn is_token_blacklisted(conn: &mut PgConnection, tokn: &str) -> Result<bool, ApiError> {
    use crate::schema::blacklisted_tokens::dsl::*;

    let result = blacklisted_tokens
        .filter(token.eq(tokn))
        .filter(expires_at.gt(Utc::now()))
        .select(token)
        .first::<String>(conn)
        .optional()
        .map_err(|e| {
            error!(
                "Error checking blacklisted token ending in ...{}: {}",
                tokn.to_string().chars().rev().take(8).collect::<String>(),
                e
            );
            ApiError::Database(e)
        })?;

    if result.is_some() {
        warn!(
            "Token ending in ...{} is blacklisted",
            tokn.to_string().chars().rev().take(8).collect::<String>()
        );
    }
    Ok(result.is_some())
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, Response> {
    let token = match extract_bearer_token(req.headers()) {
        Ok(token) => token,
        Err(error) => {
            let (status, message) = error.into();
            return Err((status, message).into_response());
        }
    };

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
            .into_response()
    })?;

    // Check if token is blacklisted - fail closed if we can't verify
    match is_token_blacklisted(&mut conn, &token) {
        Ok(true) => {
            warn!(
                "Blacklisted token used ending in ...{}",
                token.chars().rev().take(8).collect::<String>()
            );
            let error = AuthError::BlacklistedToken;
            let (status, message) = error.into();
            return Err((status, message).into_response());
        }
        Ok(false) => {
            // Token is not blacklisted, continue processing
        }
        Err(e) => {
            error!(
                "Failed to check token blacklist for token ending in ...{}: {}",
                token.chars().rev().take(8).collect::<String>(),
                e
            );
            // Fail closed: reject authentication if we can't verify blacklist status
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Authentication service temporarily unavailable".to_string(),
            )
                .into_response());
        }
    }

    let claims = match verify_token(&state, &token) {
        Ok(claims) => claims,
        Err(e) => {
            warn!(
                "JWT verification failed for token ending in ...{}: {}",
                token.chars().rev().take(8).collect::<String>(),
                e
            );
            let error = AuthError::InvalidToken("Token verification failed".to_string());
            let (status, message) = error.into();
            return Err((status, message).into_response());
        }
    };

    let now = Utc::now().timestamp() as usize;
    if claims.exp < now {
        warn!(
            "Token expired for user {}: ending in ...{}",
            claims.sub,
            token.chars().rev().take(8).collect::<String>()
        );
        let error = AuthError::InvalidToken("Token expired".to_string());
        let (status, message) = error.into();
        return Err((status, message).into_response());
    }

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
