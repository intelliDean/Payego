// Library entry point for Payego
// This exposes modules for testing while keeping main.rs as the binary entry point

use diesel::ExpressionMethods;
use diesel::QueryDsl;
mod observability;

pub mod utility;

pub use payego_primitives::error::ApiError;

use tracing::info;

use crate::utility::tasks::{build_cors, build_router, initialize_system, load_env};
use axum::extract::State;
use axum::Router;
use diesel::dsl::now;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{r2d2, PgConnection, RunQueryDsl};
use dotenvy::dotenv;
use eyre::Report;
use http::HeaderValue;
use payego_api::handlers::initialize_banks::initialize_banks;
use payego_primitives::models::app_config::AppConfig;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::schema::{blacklisted_tokens, refresh_tokens};
use secrecy::{ExposeSecret, SecretString};
use std::io::{stdout, IsTerminal};
use std::sync::Arc;
use std::time::Duration;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};
use tracing::log::debug;
use tracing::{error, warn};
use tracing_subscriber::{fmt, EnvFilter};
use crate::utility::clean_up_tasks::spawn_background_tasks;
use crate::utility::db_pool::create_db_pool;
use crate::utility::logging::setup_logging;
use crate::utility::server::serve;

pub async fn run() -> Result<(), Report> {
    // 1. Initialize logging first (so we can log everything else)
    setup_logging();

    info!("Starting Payego application...");

    // 2. Load environment variables
    load_env();

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

    // 8. Build Axum router
    let app = build_router(state.clone())?;

    // 9. Start HTTP server
    serve(app).await?;

    info!("Payego application shut down gracefully");
    Ok(())
}


//
// pub async fn run1() -> Result<(), eyre::Error> {
//     // initialize tracing
//     setup_logging();
//     info!("Starting Payego application");
//
//     // load environment variables
//     dotenv().ok();
//
//     let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
//     let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
//
//     let cors_origins = env::var("CORS_ORIGINS")
//         .unwrap_or_else(|_| "http://localhost:5173".to_string())
//         .split(',')
//         .map(|s| s.trim().to_string())
//         .collect::<Vec<String>>();
//
//     let pool = db_connection()?;
//
//     //AppState
//     let app_state = AppState::new(pool, AppConfig::from_env()?)?;
//
//     // Initialize banks
//     if let Err(e) =
//         payego_api::handlers::initialize_banks::initialize_banks(&State(app_state.clone())).await
//     {
//         error!("Failed to initialize banks: {}", e);
//         warn!("Application starting without pre-loaded banks.");
//     } else {
//         info!("Banks initialized successfully");
//     }
//
//     tokio::spawn(cleanup_expired_blacklisted_tokens(app_state.clone())); //a different threat to do this
//     tokio::spawn(cleanup_expired_refresh_tokens(app_state.clone()));
//
//
//     // Setup CORS
//     let cors = CorsLayer::new()
//         .allow_methods(Any)
//         .allow_headers(Any)
//         .allow_origin(
//             cors_origins
//                 .iter()
//                 .map(|s| s.parse::<HeaderValue>())
//                 .collect::<Result<Vec<_>, _>>()?,
//         );
//
//     let app = payego_api::app::create_router(app_state.clone()).layer(cors);
//
//     let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
//     let listener = TcpListener::bind(&addr).await?;
//
//     info!("Server running on http://{}", addr);
//     info!(
//         "Swagger UI available at http://{}/swagger-ui/index.html#/",
//         addr
//     );
//
//     axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
//         .with_graceful_shutdown(shutdown_signal())
//         .await?;
//
//     info!("Server shut down gracefully");
//     Ok(())
// }
