use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use payego_core::AppState;
use payego_primitives::schema::{blacklisted_tokens, refresh_tokens};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::error;
use tracing::log::{debug, info};

const DAILY_CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

pub fn spawn_background_tasks(state: Arc<AppState>) {
    let state_clone = state.clone();

    // Cleanup expired blacklisted tokens (daily)
    tokio::spawn(async move {
        info!("Starting daily blacklisted tokens cleanup task");
        cleanup_blacklisted_tokens(state_clone).await;
    });

    // Cleanup expired refresh tokens (daily)
    let state_clone = state.clone();
    tokio::spawn(async move {
        info!("Starting daily refresh tokens cleanup task");
        cleanup_refresh_tokens(state_clone).await;
    });

    info!("Background maintenance tasks spawned");
}

/// Improved daily cleanup (with better backoff & logging)
async fn cleanup_blacklisted_tokens(state: Arc<AppState>) {
    let mut interval = interval(DAILY_CLEANUP_INTERVAL);
    interval.tick().await;

    loop {
        interval.tick().await;

        let Ok(mut conn) = state.db.get() else {
            error!("Blacklisted token cleanup: DB connection failed");
            continue;
        };

        match diesel::delete(
            blacklisted_tokens::table.filter(blacklisted_tokens::expires_at.lt(diesel::dsl::now)),
        )
        .execute(&mut conn)
        {
            Ok(0) => debug!("No expired blacklisted tokens"),
            Ok(n) => info!("Removed {} blacklisted tokens", n),
            Err(e) => error!("Blacklisted token cleanup failed: {}", e),
        }
    }
}

async fn cleanup_refresh_tokens(state: Arc<AppState>) {
    let mut interval = interval(DAILY_CLEANUP_INTERVAL);
    interval.tick().await;

    loop {
        interval.tick().await;

        let Ok(mut conn) = state.db.get() else {
            error!("Refresh token cleanup: DB connection failed");
            continue;
        };

        match diesel::delete(
            refresh_tokens::table.filter(refresh_tokens::expires_at.lt(diesel::dsl::now)),
        )
        .execute(&mut conn)
        {
            Ok(0) => debug!("No expired refresh tokens"),
            Ok(n) => info!("Removed {} refresh tokens", n),
            Err(e) => error!("Refresh token cleanup failed: {}", e),
        }
    }
}
