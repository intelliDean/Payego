use axum::extract::State;
use axum::Router;
use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use eyre::Report;
use http::HeaderValue;
use payego_api::handlers::initialize_banks::initialize_banks;
use payego_core::app_state::AppState;
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

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

pub fn build_router(
    state: Arc<AppState>,
    metric_layer: PrometheusMetricLayer<'static>,
    metric_handle: PrometheusHandle,
) -> Result<Router, Report> {
    let cors = build_cors()?;

    Ok(
        payego_api::app::create_router(state, metric_layer, metric_handle)
            .layer(cors)
            .layer(tower_http::trace::TraceLayer::new_for_http()),
    )
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
