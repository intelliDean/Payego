use axum::{
    extract::{State, Extension},
    Json,
    http::StatusCode,
};
use diesel::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{error, info};
use utoipa::ToSchema;
use crate::{AppState, error::ApiError};
use crate::config::security_config::Claims;
use crate::schema::wallets;


#[derive(Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub balance: i64, // BIGINT for cents (e.g., 100 = $1.00)
    pub currency: String,
}



#[derive(Serialize, ToSchema)]
pub struct WalletResponse {
    pub wallets: Vec<Wallet>,
}

#[utoipa::path(
    get,
    path = "/api/wallets",
    responses(
        (status = 200, description = "List of user wallets", body = WalletResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Wallets"
)]
pub async fn get_wallets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<WalletResponse>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let conn = &mut state.db.get().map_err(|e| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let wallets = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .select(Wallet::as_select())
        .load(conn)
        .map_err(|e| {
            error!("Failed to load wallets: {}", e);
            ApiError::Database(e)
        })?;

    info!("Fetched {} wallets for user_id={}", wallets.len(), user_id);
    Ok(Json(WalletResponse { wallets }))
}