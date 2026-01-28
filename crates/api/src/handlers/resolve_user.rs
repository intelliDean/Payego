use axum::extract::Query;
use axum::{extract::Json, extract::State};
use diesel::prelude::*;
use payego_core::services::payment_service::{ApiError, AppState};
use payego_primitives::error::ApiErrorResponse;
use payego_primitives::models::dtos::wallet_dto::{ResolveUserRequest, ResolvedUser};
use payego_primitives::schema::users;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/users/resolve",
    tag = "Authentication",
    params(
        ("identifier" = String, Query, description = "Username or email")
    ),
    responses(
        ( status = 200, description = "User resolved", body = ResolvedUser),
        ( status = 404, description = "User not found — recipient with username or email not found", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = [])),
)]
pub async fn resolve_user(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ResolveUserRequest>,
) -> Result<Json<ResolvedUser>, ApiError> {
    let mut conn = state
        .db
        .get()
        .map_err(|e: r2d2::Error| ApiError::DatabaseConnection(e.to_string()))?;

    let user = users::table
        .filter(users::email.eq(&params.identifier))
        .or_filter(users::username.eq(&params.identifier))
        .select((users::id, users::email, users::username))
        .first::<ResolvedUser>(&mut conn)
        .optional()?
        .ok_or(ApiError::Database(diesel::result::Error::NotFound))?;

    Ok(Json(user))
}
