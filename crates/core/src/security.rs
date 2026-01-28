use payego_primitives::{
    error::{ApiError, AuthError},
};
use crate::app_state::AppState;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use http::HeaderMap;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub exp: i64,    // expiration time
    pub iat: i64,    // issued at
    pub iss: String,
    pub aud: String,
    pub jti: String,
}

impl Claims {
    pub fn user_id(&self) -> Result<Uuid, ApiError> {
        Uuid::parse_str(&self.sub).map_err(|e| {
            error!("Invalid user ID in claims: {}", e);
            ApiError::Auth(AuthError::InvalidToken("Invalid user ID".to_string()))
        })
    }
}

pub struct SecurityConfig;

impl SecurityConfig {
    pub fn create_token(state: &AppState, user_id: &str) -> Result<String, ApiError> {
        let now = Utc::now();

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::hours(state.config.jwt_details.jwt_expiration_hours)).timestamp(),
            iss: state.config.jwt_details.jwt_issuer.clone(),
            aud: state.config.jwt_details.jwt_audience.clone(),
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".to_string());

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(
                state
                    .config
                    .jwt_details
                    .jwt_secret
                    .expose_secret()
                    .as_bytes(),
            ),
        )
        .map_err(|e| {
            error!("JWT encoding error: {}", e);
            ApiError::Token("Token creation failed".into())
        })
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

    pub fn verify_token(state: &AppState, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[state.config.jwt_details.jwt_issuer.as_str()]);
        validation.set_audience(&[state.config.jwt_details.jwt_audience.as_str()]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(
                state
                    .config
                    .jwt_details
                    .jwt_secret
                    .expose_secret()
                    .as_bytes(),
            ),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken("Invalid or expired token".into()))
    }

    pub fn is_jti_blacklisted(conn: &mut PgConnection, jti_value: &str) -> Result<bool, ApiError> {
        use payego_primitives::schema::blacklisted_tokens::dsl::*;

        blacklisted_tokens
            .filter(jti.eq(jti_value))
            .filter(expires_at.gt(Utc::now()))
            .select(jti)
            .first::<String>(conn)
            .optional()
            .map(|res: Option<String>| res.is_some())
            .map_err(|e| {
                error!("Blacklist lookup failed for jti {}: {}", jti_value, e);
                ApiError::Database(e)
            })
    }

    pub async fn auth_middleware(
        State(state): State<Arc<AppState>>,
        mut req: Request<axum::body::Body>,
        next: Next,
    ) -> Result<Response, Response> {
        let token = Self::extract_bearer_token(req.headers())
            .map_err(|e| ApiError::from(e).into_response())?;

        let claims =
            Self::verify_token(&state, &token).map_err(|e| ApiError::from(e).into_response())?;

        let mut conn = state.db.get().map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Authentication service unavailable",
            )
                .into_response()
        })?;

        if Self::is_jti_blacklisted(&mut conn, &claims.jti).map_err(|e| e.into_response())? {
            return Err(ApiError::from(AuthError::BlacklistedToken).into_response());
        }

        req.extensions_mut().insert(claims);
        Ok(next.run(req).await)
    }
}
