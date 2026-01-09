use crate::models::models::AppState;
use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "System is healthy"),
        (status = 503, description = "System is unhealthy")
    ),
    tag = "Health"
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.db.get() {
        Ok(mut conn) => {
            // Check if we can execute a simple query
            match diesel::sql_query("SELECT 1").execute(&mut conn) {
                Ok(_) => StatusCode::OK,
                Err(_) => StatusCode::SERVICE_UNAVAILABLE,
            }
        }
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
