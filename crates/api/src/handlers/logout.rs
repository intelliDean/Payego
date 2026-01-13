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
    jti: String,
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
) -> Result<(StatusCode, Json<LogoutResponse>), ApiError> {
    use payego_primitives::schema::blacklisted_tokens::dsl::*;

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let expire_at = DateTime::<Utc>::from_timestamp(claims.exp, 0)
        .expect("exp already validated in middleware");

    let inserted = diesel::insert_into(blacklisted_tokens)
        .values(NewBlacklistedToken {
            jti: claims.jti.clone(),
            expires_at: expire_at,
        })
        .on_conflict(jti)
        .do_nothing()
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to blacklist jti {}: {}", claims.jti, e);
            ApiError::Database(e)
        })?;

    if inserted > 0 {
        info!("User {} logged out, jti {} blacklisted", claims.sub, claims.jti);
    } else {
        info!("Logout called again for already-blacklisted jti {}", claims.jti);
    }

    Ok((
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Logged out successfully".to_string(),
        }),
    ))
}
