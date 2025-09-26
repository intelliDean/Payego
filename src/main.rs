use crate::config::{
    security_config::{auth_middleware, JWTSecret},
    swagger_config::ApiDoc,
};
use crate::handlers::{
    all_banks::all_banks, bank::add_bank_account, current_user::current_user_details, login::login,
    paypal_capture::paypal_capture, paypal_order::get_paypal_order, paystack_webhook::paystack_webhook,
    register::register, stripe_webhook::stripe_webhook, top_up::top_up,
    transfer_external::external_transfer, transfer_internal::internal_transfer, withdraw::withdraw
};
use handlers::initialize_banks::initialize_banks;
use crate::models::models::AppState;
use axum::{middleware, Router};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use http::HeaderValue;
use std::{env, net::SocketAddr, sync::Arc};
use axum::extract::State;
use error::ApiError;
use tokio::{net::TcpListener, signal};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::handlers::internal_conversion::convert_currency;
use crate::handlers::resolve_account::resolve_account;
use crate::handlers::user_bank_accounts::user_bank_accounts;
use crate::handlers::user_wallets::get_wallets;
use crate::logging::setup_logging;

mod config;
mod error;
mod handlers;
mod models;
mod schema;
mod utility;
mod logging;

// Application entry point
#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Initialize tracing with environment-based log level (default: DEBUG)
    setup_logging();

    info!("Starting Payego application");

    // Load environment variables
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let db_url = "postgresql://payego_user:RRXvbF1i8QKvvxIrVNfvzPzVDy7UNJgd@dpg-d388q2ggjchc73cs1pm0-a.oregon-postgres.render.com/payego";

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let cors_origins = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    info!("cors origins: {:?}", cors_origins);

    // Setup database connection pool
    let manager = ConnectionManager::<PgConnection>::new(db_url);

    let pool = Pool::builder().max_size(10).build(manager).map_err(|e| {
        error!("Failed to create database pool: {}", e);
        eyre::eyre!("Failed to create database pool: {}", e)
    })?;

    info!("database pool: {:?}", pool);

    // Initialize AppState
    let state = Arc::new(AppState {
        db: pool,
        jwt_secret: JWTSecret::new().jwt_secret,
    });

    // Initialize banks (non-fatal failure)
    initialize_banks(State(state.clone())).await.unwrap();

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

    // Public routes (no authentication)
    let public_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/register", axum::routing::post(register))
        .route("/api/login", axum::routing::post(login))
        .route("/api/webhook/stripe", axum::routing::post(stripe_webhook))
        .route("/webhooks/paystack", axum::routing::post(paystack_webhook))
        .route("/api/bank/init", axum::routing::post(initialize_banks))
        .route("/api/banks", axum::routing::get(all_banks))
        .route("/api/resolve_account", axum::routing::get(resolve_account))
        ;

    // Protected routes (require JWT authentication)
    let protected_router = Router::new()
        .route("/api/current_user", axum::routing::get(current_user_details))
        .route("/api/bank_accounts", axum::routing::get(user_bank_accounts))
        .route("/api/wallets", axum::routing::get(get_wallets))
        .route("/api/top_up", axum::routing::post(top_up))
        .route("/api/convert_currency", axum::routing::post(convert_currency))
        .route("/api/paypal/capture", axum::routing::post(paypal_capture))
        .route(
            "/api/transfer/internal",
            axum::routing::post(internal_transfer),
        )
        .route(
            "/api/transfer/external",
            axum::routing::post(external_transfer),
        )
        .route("/api/add_bank", axum::routing::post(add_bank_account))
        .route("/api/withdraw", axum::routing::post(withdraw))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Combine routers and apply CORS
    let app = Router::new()
        .merge(public_router)
        .merge(protected_router)
        .layer(cors)
        .with_state(state);

    // Start the server
    // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    info!("About to run app");

    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&addr).await?;
    info!("Server running on http://{}", addr);
    info!(
        "Swagger UI available at http://{}/swagger-ui/index.html#/",
        addr
    );

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

// Handle Ctrl+C for graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down"),
        _ = terminate => info!("Received SIGTERM, shutting down"),
    }
}

