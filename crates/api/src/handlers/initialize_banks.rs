use axum::extract::State;
use diesel::prelude::*;
use http::StatusCode;
use payego_core::services::bank_service::BankService;
use payego_primitives::error::ApiError;
use payego_primitives::models::AppState;
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
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {
    // ensure_admin(&state)?;

    let mut conn = state.db.get().map_err(|e| {
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let initialized = BankService::initialize_banks(&state, &mut conn).await?;

    Ok(if initialized {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    })
}

// fn ensure_admin(state: &AppState) -> Result<(), ApiError> {
//     if !state.is_admin_mode {
//         return Err(ApiError::Internal("Admin access required".into()));
//     }
//     Ok(())
// }