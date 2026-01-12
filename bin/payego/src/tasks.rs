use std::env;
// Background tasks for Payego
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use eyre::Report;
use payego_primitives::models::AppState;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use tokio::signal;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{interval, Duration};
use tracing::{error, info};

pub async fn cleanup_expired_tokens(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // Run daily
    loop {
        interval.tick().await;
        let mut conn = match state.db.get() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get DB connection for token cleanup: {}", e);
                continue;
            }
        };
        if let Err(e) = diesel::delete(
            payego_primitives::schema::blacklisted_tokens::table
                .filter(payego_primitives::schema::blacklisted_tokens::expires_at.lt(Utc::now())),
        )
        .execute(&mut conn)
        {
            error!("Cleanup of expired tokens failed: {}", e);
        } else {
            info!("Successfully cleaned up expired blacklisted tokens");
        }
    }
}

pub fn db_connection() -> Result<Pool<ConnectionManager<PgConnection>>, Report> {
    let db_url = SecretString::new(Box::from(
        //to prevent accidental logging
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));

    let pool = Pool::builder()
        .max_size(50)
        .min_idle(Some(10))
        .connection_timeout(Duration::from_secs(5))
        .idle_timeout(Some(Duration::from_secs(300)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .build(ConnectionManager::<PgConnection>::new(
            db_url.expose_secret(),
        ))
        .map_err(|e| {
            error!("Failed to create database pool: {}", e);
            eyre::eyre!("Failed to create database pool: {}", e)
        })?;

    Ok(pool)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal(SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C");
        }
        _ = terminate => {
            info!("Received SIGTERM");
        }
    }

    info!("Shutdown signal received, starting graceful shutdown");
}


