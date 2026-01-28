use axum::extract::Path;
use axum::extract::{Extension, State};
use axum::Json;
use payego_core::services::bank_account_service::{ApiError, AppState, BankAccountService, Claims};
use payego_primitives::error::ApiErrorResponse;
use payego_primitives::models::dtos::bank_dto::DeleteResponse;
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/api/banks/{bank_account_id}",
    tag = "Bank",
    summary = "Delete a linked bank account",
    description = "Deletes a bank account linked to the authenticated user. This does NOT affect completed transactions.",
    operation_id = "deleteBankAccount",
    params(
        ("bank_account_id" = Uuid, Path, description = "Bank account ID to delete")
    ),
    responses(
        (status = 200, description = "Bank account deleted successfully", body = DeleteResponse),
        (status = 401, description = "Unauthorized – missing or invalid token", body = ApiErrorResponse),
        (status = 404, description = "Bank account not found or does not belong to user", body = ApiErrorResponse),
        (status = 409, description = "Conflict – bank account cannot be deleted (e.g., pending transactions)", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn delete_bank_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(bank_account_id): Path<Uuid>,
) -> Result<Json<DeleteResponse>, ApiError> {
    let user_id = claims.user_id()?;

    let res = BankAccountService::delete_bank_account(&state, user_id, bank_account_id).await?;

    Ok(Json(res))
}
