use axum::Router;
use axum_test::TestServer;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use payego::models::models::AppState;
use secrecy::SecretString;
use std::sync::Arc;
use uuid::Uuid;

pub mod fixtures;
pub mod helpers;

/// Create a test database pool
pub fn create_test_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:%40Tiptop2059!@localhost:5432/payego_test".to_string()
    });

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // Use build_unchecked if we want to allow tests to run without a live DB,
    // but here we just use builder().build() and handle it better if possible.
    // For now, let's just use build() but don't panic if it's just a unit test.
    Pool::builder()
        .max_size(5)
        .build(manager)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to create test database pool: {}. Tests requiring a database will fail.", e);
            // Return a pool anyway, it will only fail when .get() is called
            Pool::builder().build_unchecked(ConnectionManager::<PgConnection>::new("postgres://invalid"))
        })
}

/// Create a test AppState
pub fn create_test_app_state() -> Arc<AppState> {
    static INIT: std::sync::Once = std::sync::Once::new();
    let state_arc = Arc::new(AppState {
        db: create_test_db_pool(),
        jwt_secret: SecretString::from("test_secret_key_minimum_32_characters_long_for_testing"),
        stripe_secret_key: SecretString::from("sk_test_fake_key_for_testing_only"),
        app_url: "http://localhost:8080".to_string(),
        exchange_api_url: "http://localhost:8080/mock/exchange".to_string(),
        paypal_api_url: "http://localhost:8080/mock/paypal".to_string(),
        stripe_api_url: "http://localhost:8080/mock/stripe".to_string(),
        paystack_api_url: "http://localhost:8080/mock/paystack".to_string(),
        paystack_secret_key: SecretString::from("sk_test_fake_paystack_key"),
        paypal_client_id: "test_paypal_client_id".to_string(),
        paypal_secret: SecretString::from("test_paypal_secret"),
    });

    INIT.call_once(|| {
        std::env::set_var("APP_ENV", "test");
        payego::logging::setup_logging();
        let mut conn = state_arc
            .db
            .get()
            .expect("Failed to get DB connection for migrations");
        run_test_migrations(&mut conn);
        cleanup_test_db(&mut conn);
    });

    state_arc
}

/// Create a test application Router
pub fn create_test_app(state: Arc<AppState>) -> Router {
    payego::app::create_router(state)
}

/// Helper to create a test user and get its token
pub async fn create_test_user(server: &TestServer, email: &str) -> (String, String) {
    use serde_json::json;

    let response = server
        .post("/api/register")
        .json(&json!({
            "email": email,
            "password": "SecurePass123!",
            "username": Some(format!("user_{}", Uuid::new_v4()))
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    (
        body["token"].as_str().unwrap().to_string(),
        email.to_string(),
    )
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
    let _ = sql_query(
        "TRUNCATE users, wallets, transactions, bank_accounts, blacklisted_tokens CASCADE",
    )
    .execute(conn);
}
