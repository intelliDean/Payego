use payego_primitives::models::Bank;
use payego_primitives::schema::banks;
use payego_primitives::{error::ApiError, models::AppState};
use axum::{extract::State, Json};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct BankListResponse {
    pub banks: Vec<Bank>,
}

#[utoipa::path(
    get,
    path = "/api/banks",
    responses(
        (status = 200, description = "List of banks", body = BankListResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Banks"
)]
pub async fn all_banks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BankListResponse>, ApiError> {
    let mut conn = state.db.get().map_err(|e: diesel::r2d2::PoolError| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch banks from database
    let banks: Vec<Bank> = banks::table
        .filter(banks::country.eq("Nigeria")) // For now, we are only dealing with Nigerian Banks
        .load::<Bank>(&mut conn)
        .map_err(|e: diesel::result::Error| {
            error!("Failed to load banks from database: {}", e);
            ApiError::from(e)
        })?;

    if banks.is_empty() {
        error!("No banks found in database");
        return Err(ApiError::Auth("No banks found in database".to_string()));
    }

    info!("Returning {} banks from database", banks.len());
    Ok(Json(BankListResponse { banks }))
}
