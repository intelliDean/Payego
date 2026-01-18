use axum::{extract::State, Json};
use payego_primitives::{error::ApiError, models::app_state::app_state::AppState};
use std::sync::Arc;
use payego_core::services::bank_service::{BankListResponse, BankService};


#[utoipa::path(
    get,
    path = "/api/banks",
    responses(
        (status = 200, description = "List of banks", body = BankListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Banks"
)]
pub async fn all_banks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BankListResponse>, ApiError> {
    
    let response = BankService::list_banks(&state).await?;
    Ok(Json(response))
}

