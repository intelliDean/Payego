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
use diesel::dsl::now;

pub async fn cleanup_expired_blacklisted_tokens(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(24 * 60 * 60));

    // to skip immediate execution on startup
    interval.tick().await;

    loop {
        interval.tick().await;

        let mut conn = match state.db.get() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Token cleanup: failed to get DB connection: {}", e);
                continue;
            }
        };

        match diesel::delete(
            payego_primitives::schema::blacklisted_tokens::table
                .filter(payego_primitives::schema::blacklisted_tokens::expires_at.lt(now)),
        )
            .execute(&mut conn)
        {
            Ok(0) => {
                info!("Token cleanup: no expired blacklisted tokens found");
            }
            Ok(count) => {
                info!("Token cleanup: removed {} expired blacklisted tokens", count);
            }
            Err(e) => {
                error!("Token cleanup failed: {}", e);
            }
        }
    }
}

pub async fn cleanup_expired_refresh_tokens(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // daily

    // to avoid immediate run on startup
    interval.tick().await;

    loop {
        interval.tick().await;

        let mut conn = match state.db.get() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Refresh token cleanup: failed to get DB connection: {}", e);
                continue;
            }
        };

        match diesel::delete(
            payego_primitives::schema::refresh_tokens::table
                .filter(payego_primitives::schema::refresh_tokens::expires_at.lt(now)),
        )
            .execute(&mut conn)
        {
            Ok(0) => {
                info!("Refresh token cleanup: no expired tokens found");
            }
            Ok(count) => {
                info!("Refresh token cleanup: removed {} expired tokens", count);
            }
            Err(e) => {
                error!("Refresh token cleanup failed: {}", e);
            }
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


