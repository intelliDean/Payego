use axum::{extract::Query, extract::State, Extension, Json};
use payego_core::app_state::AppState;
use payego_core::repositories::audit_repository::AuditLogRepository;
use payego_core::security::Claims;
use payego_primitives::error::ApiError;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub async fn get_user_audit_logs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = claims.user_id()?;
    let limit = query.size.unwrap_or(20).min(100);
    let offset = (query.page.unwrap_or(1) - 1) * limit;

    let mut conn = state
        .db
        .get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

    let logs = AuditLogRepository::find_by_user_paginated(&mut conn, user_id, limit, offset)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "data": logs,
        "page": query.page.unwrap_or(1),
        "limit": limit
    })))
}
