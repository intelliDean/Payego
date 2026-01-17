use std::env;
// Background utility for Payego
use crate::utility::logging::setup_logging;
use axum::extract::State;
use axum::Router;
use chrono::Utc;
use diesel::{
    dsl::now,
    pg::PgConnection,
    prelude::*,
    r2d2,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use eyre::Report;
use http::HeaderValue;
use payego_api::handlers::initialize_banks::initialize_banks;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_config::AppConfig;
use payego_primitives::models::app_state::app_state::AppState;
use secrecy::{ExposeSecret, SecretString};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{interval, Duration};
use tower_http::cors::{Any, CorsLayer};
use tracing::log::warn;
use tracing::{error, info};

pub fn build_cors() -> Result<CorsLayer, Report> {
    let origins = env::var("CORS_ORIGINS").unwrap_or_else(|_| "http://localhost:5173".into());

    let allowed_origins = origins
        .split(',')
        .map(|s| s.trim().parse::<HeaderValue>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| eyre::eyre!("Invalid CORS origin: {}", e))?;

    Ok(CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(allowed_origins))
}

pub fn load_env() {
    if dotenvy::dotenv().is_ok() {
        info!("Loaded .env file");
    } else {
        info!("No .env file found, using system environment");
    }
}

pub fn build_router(state: Arc<AppState>) -> Result<Router, Report> {
    let cors = build_cors()?;

    Ok(payego_api::app::create_router(state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http()))
}

pub async fn initialize_system(state: &Arc<AppState>) {
    if let Err(e) = initialize_banks(State(state.clone())).await {
        tracing::warn!(
            "Failed to initialize banks: {}. Continuing without preloading.",
            e
        );
    } else {
        info!("System banks initialized successfully");
    }
}
