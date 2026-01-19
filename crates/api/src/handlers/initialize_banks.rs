use axum::extract::State;
use diesel::prelude::*;
use http::StatusCode;
use payego_core::services::bank_service::BankService;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;


#[utoipa::path(
    post,
    path = "/api/bank/init",
    responses(
        (status = 201, description = "Banks initialized"),
        (status = 200, description = "Banks already initialized"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn initialize_banks(
    // state: &Arc<AppState>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {

    

    let initialized = BankService::initialize_banks(&state).await?;

    Ok(if initialized {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    })
}



