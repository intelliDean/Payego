use axum::{
    extract::{Extension, State},
    Json,
};
use diesel::prelude::*;
use payego_primitives::config::security_config::Claims;
use payego_primitives::error::{ApiError, AuthError};
use payego_primitives::schema::wallets;
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::enum_types::CurrencyCode;
use payego_primitives::models::wallet::Wallet;

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletDto {
    pub id: Uuid,
    pub currency: CurrencyCode,
    pub balance: i64, // cents
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletsResponse {
    pub wallets: Vec<WalletDto>,
}

impl From<Wallet> for WalletDto {
    fn from(wallet: Wallet) -> Self {
        Self {
            id: wallet.id,
            currency: wallet.currency,
            balance: wallet.balance,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/wallets",
    responses(
        (status = 200, description = "List of user wallets", body = WalletsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Wallets"
)]
pub async fn get_wallets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<WalletsResponse>, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Auth(AuthError::InvalidToken("Invalid user ID".into())))?;

    let mut conn = state.db.get().map_err(|e| {
        error!("DB connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let wallets = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .order(wallets::created_at.asc())
        .load::<Wallet>(&mut conn)
        .map_err(ApiError::from)?;

    let response = wallets.into_iter().map(WalletDto::from).collect::<Vec<_>>();

    info!(
        user_id = %user_id,
        wallet_count = response.len(),
        "Fetched user wallets"
    );

    Ok(Json(WalletsResponse { wallets: response }))
}