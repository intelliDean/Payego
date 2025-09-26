use axum::{
    extract::{State, Extension},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::{AppState, error::ApiError};
use crate::config::security_config::Claims;
use crate::models::user_models::Bank;
use crate::schema::banks;

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
    State(state): State<Arc<AppState>>
) -> Result<Json<BankListResponse>, (StatusCode, String)> {

    let mut conn = state.db.get().map_err(|e| {
        error!("Database connection failed: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    // Fetch banks from database
    let banks: Vec<Bank> = banks::table
        .filter(banks::country.eq("Nigeria")) // For now, we are only dealing with Nigerian Banks
        .load::<Bank>(&mut conn)
        .map_err(|e| {
            error!("Failed to load banks from database: {}", e);
            ApiError::Database(e)
        })?;

    if banks.is_empty() {
        error!("No banks found in database");
        return Err(ApiError::Auth("No banks found in database".to_string()).into());
    }

    info!("Returning {} banks from database", banks.len());
    Ok(Json(BankListResponse { banks }))
}