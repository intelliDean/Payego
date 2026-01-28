mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use common::{create_test_app, create_test_app_state};
use diesel::prelude::*;
use payego_primitives::schema::verification_tokens::dsl::*;
use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_full_email_verification_flow() {
    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();

    let email_str = format!("verify_{}@example.com", uuid::Uuid::new_v4());

    // 1. Register user
    let res = server
        .post("/api/auth/register")
        .json(&json!({
            "email": email_str,
            "password": "SecurePass123!",
            "username": Some(format!("user_{}", uuid::Uuid::new_v4()))
        }))
        .await;
    res.assert_status(StatusCode::CREATED);

    // 2. Check user status (should be unverified)
    let login_res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": email_str,
            "password": "SecurePass123!"
        }))
        .await;
    login_res.assert_status(StatusCode::OK);
    let login_body: serde_json::Value = login_res.json();
    let token = login_body["token"].as_str().unwrap();

    // Get user info to check verification
    let user_res = server
        .get("/api/user/current")
        .add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        )
        .await;
    user_res.assert_status(StatusCode::OK);
    let user_body: serde_json::Value = user_res.json();
    assert!(user_body["email_verified_at"].is_null());

    // 3. Manually create a known verification token for testing
    let raw_test_token = "test-token-uuid-12345";
    let hashed_test_token =
        payego_core::services::auth_service::verification::VerificationService::hash_token(
            raw_test_token,
        );

    let mut conn = state.db.get().unwrap();

    // Find the user ID we just created
    use payego_primitives::schema::users::dsl::*;
    let user_id_val: uuid::Uuid = users
        .filter(email.eq(email_str))
        .select(id)
        .first(&mut conn)
        .expect("User should exist");

    // Insert our known token
    use payego_core::repositories::verification_repository::VerificationRepository;
    use payego_primitives::models::entities::verification_token::NewVerificationToken;

    VerificationRepository::delete_for_user(&mut conn, user_id_val).unwrap();
    VerificationRepository::create(
        &mut conn,
        NewVerificationToken {
            user_id: user_id_val,
            token_hash: hashed_test_token,
            expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
        },
    )
    .unwrap();

    // 4. Verify email using the RAW token
    let verify_res = server
        .get(&format!("/api/auth/verify-email?token={}", raw_test_token))
        .await;
    verify_res.assert_status(StatusCode::OK);

    // 5. Check user status again (should be verified)
    let user_res_after = server
        .get("/api/user/current")
        .add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        )
        .await;
    user_res_after.assert_status(StatusCode::OK);
    let user_body_after: serde_json::Value = user_res_after.json();
    println!("DBG: User body after verification: {:#?}", user_body_after);
    assert!(!user_body_after["email_verified_at"].is_null());
}

#[tokio::test]
#[serial]
async fn test_resend_verification_email() {
    let state = create_test_app_state();
    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();

    let email_str = format!("resend_{}@example.com", uuid::Uuid::new_v4());

    // Register
    server
        .post("/api/auth/register")
        .json(&json!({
            "email": email_str,
            "password": "SecurePass123!",
            "username": Some(format!("user_{}", uuid::Uuid::new_v4()))
        }))
        .await
        .assert_status(StatusCode::CREATED);

    // Login to get token
    let login_res = server
        .post("/api/auth/login")
        .json(&json!({
            "email": email_str,
            "password": "SecurePass123!"
        }))
        .await;
    let auth_token = login_res.json::<serde_json::Value>()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Resend
    let resend_res = server
        .post("/api/auth/resend-verification")
        .add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", auth_token),
        )
        .await;
    resend_res.assert_status(StatusCode::OK);

    // Verify a new token was created
    let mut conn = state.db.get().unwrap();
    let count: i64 = verification_tokens.count().get_result(&mut conn).unwrap();
    assert!(count >= 1);
}
