use axum::{extract::State, Json};
use diesel::prelude::*;
use payego_primitives::schema::banks;
use payego_primitives::{error::ApiError, models::app_state::app_state::AppState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use payego_primitives::models::bank::Bank;

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
    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    const COUNTRY: &str = "Nigeria";

    let banks: Vec<Bank> = banks::table
        .filter(banks::country.eq(COUNTRY))
        .load::<Bank>(&mut conn)
        .map_err(|e| {
            error!("Failed to load banks from database: {}", e);
            ApiError::from(e)
        })?;

    info!("Returning {} banks", banks.len());

    Ok(Json(BankListResponse { banks }))
}
