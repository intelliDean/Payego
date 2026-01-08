use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel::prelude::*;
use payego::models::models::AppState;
use std::sync::Arc;

pub mod fixtures;
pub mod helpers;

/// Create a test database pool
pub fn create_test_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://payego_user:password@localhost/payego_test".to_string());
    
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // Use build_unchecked if we want to allow tests to run without a live DB,
    // but here we just use builder().build() and handle it better if possible.
    // For now, let's just use build() but don't panic if it's just a unit test.
    Pool::builder()
        .max_size(1)
        .build(manager)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to create test database pool: {}. Tests requiring a database will fail.", e);
            // Return a pool anyway, it will only fail when .get() is called
            Pool::builder().build_unchecked(ConnectionManager::<PgConnection>::new("postgres://invalid"))
        })
}

/// Create a test AppState
pub fn create_test_app_state() -> Arc<AppState> {
    Arc::new(AppState {
        db: create_test_db_pool(),
        jwt_secret: "test_secret_key_minimum_32_characters_long_for_testing".to_string(),
        stripe_secret_key: "sk_test_fake_key_for_testing_only".to_string(),
        app_url: "http://localhost:8080".to_string(),
    })
}

/// Run database migrations for tests
#[allow(dead_code)]
pub fn run_test_migrations(conn: &mut PgConnection) {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}

/// Clean up test database
#[allow(dead_code)]
pub fn cleanup_test_db(conn: &mut PgConnection) {
    use diesel::sql_query;
    
    // Truncate all tables
    let _ = sql_query("TRUNCATE users, wallets, transactions, bank_accounts, blacklisted_tokens CASCADE")
        .execute(conn);
}
