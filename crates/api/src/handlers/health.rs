use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use tracing::log::error;

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
                Err(e) => {
                    error!("Health check DB query failed: {}", e);
                    StatusCode::SERVICE_UNAVAILABLE
                },
            }
        }
        Err(e) => {
            error!("Health check DB connection failed: {}", e);
            return StatusCode::SERVICE_UNAVAILABLE
        },
    }
}
