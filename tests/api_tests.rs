mod common;

use axum_test::TestServer;
use common::{create_test_app, create_test_app_state};
use http::StatusCode;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_user_registration_success() {
    let state = create_test_app_state();

    // Run migrations and cleanup
    {
        let mut conn = state.db.get().expect("Failed to get DB connection");
        common::run_test_migrations(&mut conn);
        common::cleanup_test_db(&mut conn);
    }

    let app = create_test_app(state);
    let server = TestServer::new(app).unwrap();

    let email = format!("test_{}@example.com", Uuid::new_v4());
    let response = server
        .post("/api/register")
        .json(&json!({
            "email": email,
            "password": "SecurePass123!",
            "username": Some(format!("user_{}", Uuid::new_v4()))
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert!(body["token"].is_string());
    assert_eq!(body["user_email"], email);
}

#[tokio::test]
#[serial]
async fn test_duplicate_email_rejected() {
    let state = create_test_app_state();

    // Run migrations and cleanup
    {
        let mut conn = state.db.get().expect("Failed to get DB connection");
        common::run_test_migrations(&mut conn);
        common::cleanup_test_db(&mut conn);
    }

    let app = create_test_app(state);
    let server = TestServer::new(app).unwrap();

    let email = format!("dup_{}@example.com", Uuid::new_v4());
    let reg_data = json!({
        "email": email,
        "password": "SecurePass123!",
        "username": Some("duplicate_user")
    });

    // First registration
    server
        .post("/api/register")
        .json(&reg_data)
        .await
        .assert_status(StatusCode::CREATED);

    // Attempt duplicate
    let response = server.post("/api/register").json(&reg_data).await;

    // Status should be BAD_REQUEST (400)
    response.assert_status(StatusCode::BAD_REQUEST);
}
