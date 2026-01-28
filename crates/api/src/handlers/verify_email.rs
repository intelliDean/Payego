use axum::{extract::Query, extract::State, Extension, Json};
use payego_core::app_state::AppState;
use payego_core::security::Claims;
use payego_core::services::auth_service::verification::VerificationService;
use payego_primitives::error::{ApiError, AuthError};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    VerificationService::verify_email(&state, &query.token).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Email verified successfully"
    })))
}

pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = claims.user_id()?;

    let mut conn = state
        .db
        .get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
    let user =
        payego_core::repositories::user_repository::UserRepository::find_by_id(&mut conn, user_id)?
            .ok_or_else(|| ApiError::Auth(AuthError::InternalError("User not found".into())))?;

    VerificationService::send_verification_email(&state, user_id, &user.email).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Verification email resent"
    })))
}
