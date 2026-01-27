use axum::{extract::State, Json};
use payego_core::services::bank_service::{ApiError, AppState, BankListResponse, BankService};
use payego_primitives::error::ApiErrorResponse;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/banks/all",
    summary = "Get list of supported banks",
    description = "Retrieves a list of banks available in the configured country. \
                   Useful for populating bank selection dropdowns during transfers or account verification.",
    operation_id = "getAllBanks",
    tags = ["Bank"],
    responses(
        (status = 200, description = "Successfully retrieved list of banks", body = BankListResponse),
        (status = 400, description = "Invalid query parameters (e.g. unknown country code)", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized — missing or invalid authentication", body = ApiErrorResponse),
        (status = 403, description = "Forbidden — insufficient permissions", body = ApiErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ApiErrorResponse),
        (status = 500, description = "Internal server error — something went wrong on our side", body = ApiErrorResponse),
    ),
    security(()),
)]
pub async fn all_banks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BankListResponse>, ApiError> {
    let response = BankService::list_banks(&state).await?;
    Ok(Json(response))
}
