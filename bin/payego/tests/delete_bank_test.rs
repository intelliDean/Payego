use axum_test::TestServer;
use diesel::prelude::*;
use http::StatusCode;

use payego_primitives::models::entities::enum_types::CurrencyCode;
use payego_primitives::schema::{bank_accounts, banks, users};
use serial_test::serial;
use uuid::Uuid;


mod common;
use common::{create_test_app, create_test_app_state};

#[tokio::test]
#[serial]
async fn test_delete_bank_account_success() {
    let state = create_test_app_state();

    // 1. Run migrations and cleanup
    {
        let mut conn = state.db.get().expect("Failed to get DB connection");
        common::run_test_migrations(&mut conn);
        common::cleanup_test_db(&mut conn);
    }

    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();

    // 2. Create User and Login (Mocking login by manually creating token or user)
    // Actually, `create_test_app` might not mock auth.
    // We can insert a user and generate a token if needed, OR just rely on internal helpers if they exist.
    // However, the delete endpoint requires authentication.
    // Let's see how we can authenticate.
    // `api_tests.rs` used `server.post("/api/auth/register")`. We can do that to get a token.

    let email = format!("delete_test_{}@example.com", Uuid::new_v4());
    let register_response = server
        .post("/api/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": "SecurePass123!",
            "username": Some(format!("user_{}", Uuid::new_v4()))
        }))
        .await;
    
    register_response.assert_status(StatusCode::CREATED);
    let token = register_response.json::<serde_json::Value>()["token"].as_str().unwrap().to_string();
    
    // We need the user ID. We can get it from the DB or the response (if it returns it).
    // The register response has `user_id` or similar? `api_tests.rs` checked `user_email`.
    // Let's just query the DB for the user.
    let mut conn = state.db.get().unwrap();
    let user = users::table
        .filter(users::email.eq(&email))
        .first::<payego_primitives::models::user::User>(&mut conn)
        .unwrap();
    let user_id = user.id;

    // 3. Insert a Bank Account manually
    // First, we need a Bank in the `banks` table.
    let bank_id = 999;
    diesel::insert_into(banks::table)
        .values((
            banks::id.eq(bank_id),
            banks::name.eq("Test Delete Bank"),
            banks::code.eq("999"),
            banks::currency.eq(CurrencyCode::NGN),
            banks::country.eq("Nigeria"),
        ))
        .execute(&mut conn)
        .unwrap();

    let bank_account_id = Uuid::new_v4();
    diesel::insert_into(bank_accounts::table)
        .values((
            bank_accounts::id.eq(bank_account_id),
            bank_accounts::user_id.eq(user_id),
            bank_accounts::bank_name.eq("Test Delete Bank"),
            bank_accounts::account_number.eq("1234567890"),
            bank_accounts::account_name.eq(Some("Test Account".to_string())),
            bank_accounts::bank_code.eq("999"),
        ))
        .execute(&mut conn)
        .unwrap();

    // 4. Call DELETE endpoint
    let response = server
        .delete(&format!("/api/banks/{}", bank_account_id))
        .add_header("Authorization", &format!("Bearer {}", token))
        .await;

    // 5. Verify Response
    response.assert_status(StatusCode::OK);

    // 6. Verify DB Deletion
    let count = bank_accounts::table
        .filter(bank_accounts::id.eq(bank_account_id))
        .count()
        .get_result::<i64>(&mut conn)
        .unwrap();
    
    assert_eq!(count, 0, "Bank account should be deleted");
}
