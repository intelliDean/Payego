// use http_body_util::BodyExt;
use crate::error::ApiError;
use crate::models::user_models::{AppState, ErrorResponse};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use headers::HeaderMapExt;
use headers::{Authorization, authorization::Bearer};
use http::HeaderMap;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
    errors::Error as JwtError,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::os::linux::raw::stat;
use std::sync::Arc;
use tower::ServiceExt;
use tracing::error;
use tracing::log::warn;

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
        .unwrap_or_else(|_| "48".to_string())
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

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| {
        error!("JWT encoding error: {}", e);
        ApiError::Token(format!("Token creation failed: {}", e))
    })
}


#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidFormat,
    InvalidToken(String),
    InternalError(String),
}

// Implement Display for better error messages
impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "Authorization header required"),
            AuthError::InvalidFormat => write!(f, "Invalid Authorization format"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}
//
// // Convert to HTTP response
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
            AuthError::InvalidToken(msg) => (
                StatusCode::UNAUTHORIZED,
                format!("Invalid token: {}", msg),
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

    // Validate token is not empty
    if token.is_empty() {
        return Err(AuthError::InvalidFormat);
    }

    Ok(token.to_string())
}

// Enhanced verify_token function
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


pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, Response> {  // Change return type to Result<Response, Response>
    // Extract token from headers
    let token = match extract_bearer_token(req.headers()) {
        Ok(token) => token,
        Err(error) => {
            let (status, message) = error.into();
            return Err((status, message).into_response());
        }
    };

    // Verify token
    let claims = match verify_token(&state, &token) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("JWT verification failed: {}", e);
            let error = AuthError::InvalidToken("Token verification failed".to_string());
            let (status, message) = error.into();
            return Err((status, message).into_response());
        }
    };

    // Check if token is expired
    let now = Utc::now().timestamp() as usize;

    if claims.exp < now {
        warn!("Token expired for user: {}", claims.sub);
        let error = AuthError::InvalidToken("Token expired".to_string());
        let (status, message) = error.into();
        return Err((status, message).into_response());
    }

    // Store claims in request extensions
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}