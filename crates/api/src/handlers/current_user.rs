use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::schema::{users, wallets};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use payego_primitives::models::enum_types::CurrencyCode;

#[derive(Serialize, ToSchema)]
pub struct Wallet {
    pub currency: CurrencyCode,
    pub balance: i64,
}

#[derive(Serialize, ToSchema)]
pub struct CurrentUserResponse {
    pub email: String,
    pub wallets: Vec<Wallet>,
}

#[utoipa::path(
    get,
    path = "/api/current_user",
    responses(
        (status = 200, description = "User data retrieved successfully", body = CurrentUserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "User"
)]
pub async fn current_user_details(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<CurrentUserResponse>, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        ApiError::Auth(AuthError::InvalidToken("Invalid subject".into()))
    })?;

    let mut conn = state.db.get().map_err(|e| {
        error!("DB connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch user email (fail fast if user doesn't exist)
    let email = users::table
        .find(user_id)
        .select(users::email)
        .first::<String>(&mut conn)
        .optional()
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::Auth(AuthError::InvalidToken(
                "User no longer exists".into(),
            ))
        })?;

    let wallets = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .select((wallets::currency, wallets::balance))
        .load::<(CurrencyCode, i64)>(&mut conn)
        .map_err(ApiError::from)?
        .into_iter()
        .map(|(currency, balance)| Wallet { currency, balance })
        .collect();

    Ok(Json(CurrentUserResponse { email, wallets }))
}