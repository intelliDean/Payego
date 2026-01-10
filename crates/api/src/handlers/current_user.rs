use payego_primitives::config::security_config::Claims;
use payego_primitives::error::ApiError;
use payego_primitives::schema::{users, wallets};
use payego_primitives::models::AppState;
use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct Wallet {
    pub currency: String,
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
    // Parse user ID from JWT claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID in JWT: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let conn = &mut state.db.get().map_err(|e: diesel::r2d2::PoolError| {
        error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch user email
    let user = users::table
        .filter(users::id.eq(user_id))
        .select(users::email)
        .first::<String>(conn)
        .map_err(|e: diesel::result::Error| {
            error!("Failed to fetch user: {}", e);
            ApiError::from(e)
        })?;

    // Fetch wallets
    let wallets = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .select((wallets::currency, wallets::balance))
        .load::<(String, i64)>(conn)
        .map_err(|e: diesel::result::Error| {
            error!("Failed to fetch wallets: {}", e);
            ApiError::from(e)
        })?
        .into_iter()
        .map(|(currency, balance)| Wallet { currency, balance })
        .collect::<Vec<Wallet>>();

    info!("Fetched user data for user_id: {}", user_id);

    Ok(Json(CurrentUserResponse {
        email: user,
        wallets,
    }))
}
