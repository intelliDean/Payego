use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use payego_primitives::config::security_config::{verify_token, Claims};
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::AppState;
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info, warn};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    message: String,
}

#[derive(Insertable)]
#[diesel(table_name = payego_primitives::schema::blacklisted_tokens)]
struct NewBlacklistedToken {
    token: String,
    expires_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/logout",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: axum::http::HeaderMap,
) -> Result<(StatusCode, Json<LogoutResponse>), ApiError> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ").map(|t| t.trim()))
        .ok_or_else(|| {
            error!("Missing or invalid Authorization header");
            ApiError::Auth(AuthError::InvalidToken("Missing or invalid Authorization header".to_string()))
        })?;

    let verified_claims = verify_token(&state, token).map_err(|e| {
        error!(
            "Token verification failed during logout for token ending in ...{}: {}",
            token.chars().rev().take(8).collect::<String>(),
            e
        );
        ApiError::Auth(AuthError::InvalidToken("Invalid token".to_string()))
    })?;

    if verified_claims.sub != claims.sub {
        error!(
            "Token user mismatch: expected {}, got {}",
            claims.sub, verified_claims.sub
        );
        return Err(ApiError::Auth(AuthError::InvalidToken("Token user mismatch".to_string())));
    }

    let mut conn = state.db.get().map_err(|e: r2d2::Error| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let expires_at =
        chrono::DateTime::<Utc>::from_timestamp(claims.exp as i64, 0).ok_or_else(|| {
            error!("Invalid expiration timestamp: {}", claims.exp);
            ApiError::Auth(AuthError::InvalidToken("Invalid expiration timestamp".to_string()))
        })?;

    let result = diesel::insert_into(payego_primitives::schema::blacklisted_tokens::table)
        .values(NewBlacklistedToken {
            token: token.to_string(),
            expires_at,
        })
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .map_err(|e| {
            error!(
                "Failed to blacklist token ending in ...{}: {}",
                token.chars().rev().take(8).collect::<String>(),
                e
            );
            ApiError::from(e)
        })?;

    if result > 0 {
        info!(
            "User {} logged out successfully, token ending in ...{} blacklisted",
            claims.sub,
            token.chars().rev().take(8).collect::<String>()
        );
    } else {
        warn!(
            "Token ending in ...{} already blacklisted or not inserted",
            token.chars().rev().take(8).collect::<String>()
        );
    }

    Ok((
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Logged out successfully".to_string(),
        }),
    ))
}
