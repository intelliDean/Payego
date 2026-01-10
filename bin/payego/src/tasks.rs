// Background tasks for Payego
use payego_primitives::models::AppState;
use chrono::Utc;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::sync::Arc;
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
