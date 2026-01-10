// Library entry point for Payego
// This exposes modules for testing while keeping main.rs as the binary entry point

pub mod logging;
mod observability;

pub mod tasks;

pub use payego_primitives::error::ApiError;
pub use payego_primitives::models::AppState;

use tokio::signal;
use tracing::info;

use crate::logging::setup_logging;
use crate::tasks::{cleanup_expired_tokens, db_connection, shutdown_signal};
use axum::extract::State;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use http::HeaderValue;
use secrecy::{ExposeSecret, SecretString};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, warn};

pub async fn run() -> Result<(), eyre::Error> {
    // initialize tracing
    setup_logging();
    info!("Starting Payego application");

    // load environment variables
    dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let cors_origins = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    let pool = db_connection()?;

    //AppState
    let app_state = AppState::new(pool)?;

    // Initialize banks
    if let Err(e) =
        payego_api::handlers::initialize_banks::initialize_banks(State(app_state.clone())).await
    {
        error!("Failed to initialize banks: {}", e);
        warn!("Application starting without pre-loaded banks.");
    } else {
        info!("Banks initialized successfully");
    }

    tokio::spawn(cleanup_expired_tokens(app_state.clone())); //a different threat to do this

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(
            cors_origins
                .iter()
                .map(|s| s.parse::<HeaderValue>())
                .collect::<Result<Vec<_>, _>>()?,
        );

    let app = payego_api::app::create_router(app_state.clone()).layer(cors);

    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&addr).await?;

    info!("Server running on http://{}", addr);
    info!(
        "Swagger UI available at http://{}/swagger-ui/index.html#/",
        addr
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}