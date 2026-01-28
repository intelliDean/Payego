mod common;

use axum_test::TestServer;
use common::{create_test_app, create_test_app_state, create_test_user};
use diesel::prelude::*;
use payego_primitives::models::entities::audit_log::AuditLog;
use payego_primitives::schema::audit_logs;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_audit_log_on_registration() {
    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();

    let email = format!("audit_test_{}@example.com", uuid::Uuid::new_v4());

    // Perform registration
    let _ = create_test_user(&server, &email).await;

    // Verify audit log
    let mut conn = state.db.get().unwrap();
    let logs: Vec<AuditLog> = audit_logs::table
        .filter(audit_logs::event_type.eq("auth.register"))
        .load(&mut conn)
        .unwrap();

    assert!(!logs.is_empty());
    let log = logs
        .iter()
        .find(|l| l.metadata["email"] == email)
        .expect("Audit log for registration not found");
    assert_eq!(log.event_type, "auth.register");
}

#[tokio::test]
#[serial]
async fn test_audit_log_on_login() {
    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();

    let email = format!("login_audit_{}@example.com", uuid::Uuid::new_v4());
    let _ = create_test_user(&server, &email).await;

    // Perform login
    let response = server
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": email,
            "password": "SecurePass123!"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::OK);

    // Verify audit log
    let mut conn = state.db.get().unwrap();
    let logs: Vec<AuditLog> = audit_logs::table
        .filter(audit_logs::event_type.eq("auth.login.success"))
        .load(&mut conn)
        .unwrap();

    assert!(!logs.is_empty());
}
