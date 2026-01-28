// Library entry point for Payego
// This exposes modules for testing while keeping main.rs as the binary entry point
mod observability;

pub mod utility;

pub use payego_primitives::error::ApiError;

use crate::utility::clean_up_tasks::spawn_background_tasks;
use crate::utility::db_pool::create_db_pool;
use crate::utility::logging::setup_logging;
use crate::utility::server::serve;
use crate::utility::tasks::{build_router, initialize_system, load_env};
use eyre::Report;
use payego_primitives::models::app_config::AppConfig;
use payego_core::app_state::AppState;
use tracing::info;

pub async fn run() -> Result<(), Report> {
    // 1. Load environment variables
    load_env();

    // 2. Initialize logging first (so we can log everything else)
    setup_logging();

    info!("Starting Payego application...");

    // 3. Load configuration
    let config = AppConfig::from_env()?;

    // 4. Create database connection pool
    let pool = create_db_pool()?;

    // 5. Build application state
    let state = AppState::new(pool, config)?;

    // 6. Perform one-time system initialization
    initialize_system(&state).await;

    // 7. Start background maintenance tasks
    spawn_background_tasks(state.clone());

    // 8. Initialize metrics
    let (metric_layer, metric_handle) = crate::observability::metrics::setup_metrics();

    // 9. Build Axum router
    let app = build_router(state.clone(), metric_layer, metric_handle)?;

    // 10. Start HTTP server
    serve(app).await?;

    info!("Payego application shut down gracefully");
    Ok(())
}
