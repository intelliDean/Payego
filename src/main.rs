use axum::extract::State;
use chrono::Utc;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use dotenvy::dotenv;
use http::HeaderValue;
use payego::config::security_config::JWTSecret;
use payego::handlers::initialize_banks::initialize_banks;
use payego::logging::setup_logging;
use payego::models::models::AppState;
use secrecy::SecretString;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::time::{interval, Duration};
use tokio::{net::TcpListener, signal};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // initialize tracing
    setup_logging();
    info!("Starting Payego application");

    // load environment variables
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let cors_origins = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    // database connection pool
    let manager = ConnectionManager::<PgConnection>::new(db_url);

    let pool = Pool::builder()
        .max_size(50)
        .min_idle(Some(10))
        .connection_timeout(Duration::from_secs(5))
        .idle_timeout(Some(Duration::from_secs(300)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .build(manager)
        .map_err(|e| {
            error!("Failed to create database pool: {}", e);
            eyre::eyre!("Failed to create database pool: {}", e)
        })?;

    //AppState
    let state = Arc::new(AppState {
        db: pool,
        jwt_secret: SecretString::new(JWTSecret::new().jwt_secret.into()),
        stripe_secret_key: SecretString::new(
            env::var("STRIPE_SECRET_KEY")
                .map_err(|e| {
                    error!("STRIPE_SECRET_KEY environment variable not set: {}", e);
                    eyre::eyre!("STRIPE_SECRET_KEY environment variable must be set")
                })?
                .into(),
        ),
        paystack_secret_key: SecretString::new(
            env::var("PAYSTACK_SECRET_KEY")
                .map_err(|e| {
                    error!("PAYSTACK_SECRET_KEY environment variable not set: {}", e);
                    eyre::eyre!("PAYSTACK_SECRET_KEY environment variable must be set")
                })?
                .into(),
        ),
        app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        exchange_api_url: env::var("EXCHANGE_API_URL")
            .unwrap_or_else(|_| "https://api.exchangerate-api.com/v4/latest".to_string()),
        paypal_api_url: env::var("PAYPAL_API_URL")
            .unwrap_or_else(|_| "https://api-m.sandbox.paypal.com".to_string()),
        stripe_api_url: env::var("STRIPE_API_URL")
            .unwrap_or_else(|_| "https://api.stripe.com".to_string()),
        paystack_api_url: env::var("PAYSTACK_API_URL")
            .unwrap_or_else(|_| "https://api.paystack.co".to_string()),
        paypal_client_id: env::var("PAYPAL_CLIENT_ID").unwrap_or_default(),
        paypal_secret: SecretString::new(env::var("PAYPAL_SECRET").unwrap_or_default().into()),
    });

    // Initialize banks
    if let Err((status, message)) = initialize_banks(State(state.clone())).await {
        error!("Failed to initialize banks ({}): {}", status, message);
        warn!("Application starting without pre-loaded banks.");
    } else {
        info!("Banks initialized successfully");
    }

    tokio::spawn(cleanup_expired_tokens(state.clone()));

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

    let app = payego::app::create_router(state.clone()).layer(cors);

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

async fn cleanup_expired_tokens(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // Run daily
    loop {
        interval.tick().await;
        // Db connection handling...
        // Assuming simplified loop logic here for brevity as main.rs content
        // The previous implementation was correct
        let mut conn = match state.db.get() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get DB connection: {}", e);
                continue;
            }
        };
        if let Err(e) = diesel::delete(
            payego::schema::blacklisted_tokens::table
                .filter(payego::schema::blacklisted_tokens::expires_at.lt(Utc::now())),
        )
        .execute(&mut conn)
        {
            error!("Cleanup failed: {}", e);
        }
    }
}
