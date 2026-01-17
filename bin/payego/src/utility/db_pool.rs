use std::env;
use std::time::Duration;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use eyre::Report;
use secrecy::{ExposeSecret, SecretString};
use tracing::info;

pub fn create_db_pool() -> Result<Pool<ConnectionManager<PgConnection>>, Report> {
    let db_url = SecretString::new(Box::from(
        env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));

    let manager = ConnectionManager::<PgConnection>::new(db_url.expose_secret());

    let pool = Pool::builder()
        .max_size(50) // adjust based on traffic
        .min_idle(Some(5))
        .connection_timeout(Duration::from_secs(8))
        .idle_timeout(Some(Duration::from_secs(300)))
        .max_lifetime(Some(Duration::from_secs(1800))) // 30 minutes
        .test_on_check_out(true) // recommended for production
        .build(manager)?;

    info!("PostgreSQL connection pool created (max_size: 50)");

    Ok(pool)
}
