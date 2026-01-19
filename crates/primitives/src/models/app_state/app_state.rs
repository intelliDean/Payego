use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use crate::models::app_config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub http_client: Client,
    pub config: AppConfig,
}

use eyre::Result;

impl AppState {
    pub fn new(db: DbPool, config: AppConfig) -> Result<Arc<Self>> {
        let http = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Arc::new(Self { db, http_client: http, config }))
    }
}
