use payego_primitives::error::ApiErrorResponse;
use axum::{
    extract::{Extension, State},
    Json,
};
use payego_core::services::auth_service::user::{
    ApiError, AppState, Claims, CurrentUserResponse, UserService,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/user/current",
    summary = "Get current authenticated user details",
    description = "Retrieves profile information for the currently authenticated user based on the JWT bearer token. \
                   Returns user data including ID, email, name, etc. \
                   Requires a valid authentication token.",
    operation_id = "getCurrentUser",
    tags = ["Authentication"],
    responses(
        (status = 200,description = "Successfully retrieved current user data",body = CurrentUserResponse,),
        (status = 401,description = "Unauthorized – missing, invalid, or expired token",body = ApiErrorResponse,),
        (status = 500,description = "Internal server error – unexpected issue on server side",body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn current_user_details(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<CurrentUserResponse>, ApiError> {
    let user_id = claims.user_id()?;

    let response = UserService::current_user_summary(&state, user_id).await?;

    Ok(Json(response))
}
