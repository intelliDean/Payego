use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use payego_primitives::models::app_state::AppState;
use payego_primitives::models::dtos::auth_dto::HealthStatus;
use std::sync::Arc;
use tracing::log::error;

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health",
    summary = "Health check endpoint",
    description = "Simple health check that returns the current operational status of the API service. \
                   Returns 200 OK when the service is healthy and able to handle requests. \
                   Returns 503 Service Unavailable when the service is unhealthy (e.g. database down, critical dependency failure). \
                   This endpoint is **public** (no authentication required)
                   Response time should be very fast (< 100–200 ms) — avoid heavy operations.",
    operation_id = "healthCheck",
    responses(
        ( status = 200, description = "Service is healthy and operational", body = HealthStatus),
        ( status = 503, description = "Service is unhealthy – one or more critical components are down \
                           (database, cache, payment gateway, etc.)", body = HealthStatus),
    ),
    security(()),
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthStatus> {
    match state.db.get() {
        Ok(mut conn) => {
            // Check if we can execute a simple query
            match diesel::sql_query("SELECT 1").execute(&mut conn) {
                Ok(_) => Json(HealthStatus {
                    status: StatusCode::OK.to_string(),
                    message: "API is healthy".to_string(),
                }),
                Err(e) => {
                    error!("Health check DB query failed: {}", e);
                    Json(HealthStatus {
                        status: StatusCode::SERVICE_UNAVAILABLE.to_string(),
                        message: "Health check DB query failed".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            error!("Health check DB connection failed: {}", e);
            Json(HealthStatus {
                status: StatusCode::SERVICE_UNAVAILABLE.to_string(),
                message: "Health check DB connection failed".to_string(),
            })
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, ToSchema)]
// pub struct HealthStatus {
//     pub status: String,
//     pub message: String,
// }
