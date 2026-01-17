use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, Connection, PgConnection};
use dotenvy::dotenv;
use payego_core::services::bank_service::BankService;
use payego_primitives::models::AppState;
use payego_primitives::models::BankRequest;
use payego_primitives::schema::{bank_accounts, users};
use serde_json::json;
use serial_test::serial;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn get_test_pool() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to create pool")
}

#[tokio::test]
#[serial]
async fn test_add_bank_account_success() {
    // 1. Setup WireMock
    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    // Mock Paystack Resolve Account
    Mock::given(method("GET"))
        .and(path("/bank/resolve"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": true,
            "message": "Account number resolved",
            "data": {
                "account_number": "0001234567",
                "account_name": "Test User Account"
            }
        })))
        .mount(&mock_server)
        .await;

    // Mock Paystack Create Recipient
    Mock::given(method("POST"))
        .and(path("/transferrecipient"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "status": true,
            "message": "Recipient created",
            "data": {
                "recipient_code": "RCP_123456",
                "details": {
                    "account_name": "Test User Account"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    // 2. Setup DB
    let pool = get_test_pool();
    let conn = &mut pool.get().unwrap();

    let user_id = Uuid::new_v4();
    let email = format!("test_bank_{}@example.com", user_id);

    // Insert User
    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::email.eq(email),
            users::password_hash.eq("hash"),
        ))
        .execute(conn)
        .unwrap();

    use secrecy::SecretString;
    // 3. Setup AppState
    let state = Arc::new(AppState {
        db: pool.clone(),
        jwt_secret: SecretString::from("secret"),
        jwt_expiration_hours: 2,
        jwt_issuer: "paye".to_string(),
        jwt_audience: "paye_api".to_string(),
        client: Default::default(),
        stripe_secret_key: SecretString::from("sk_test"),
        app_url: "http://localhost:8080".to_string(),
        exchange_api_url: "http://unused".to_string(),
        paypal_api_url: "http://unused".to_string(),
        stripe_api_url: "http://unused".to_string(),
        stripe_webhook_secret: Default::default(),
        paystack_api_url: base_url.clone(),
        paystack_secret_key: SecretString::from("sk_test_paystack"),
        paystack_webhook_secret: Default::default(),
        paypal_client_id: "test_client_id".to_string(),
        paypal_secret: SecretString::from("test_secret"),
    });

    // 4. Call Service
    let req = BankRequest {
        bank_name: "Test Bank".to_string(),
        bank_code: "057".to_string(),
        account_number: "0001234567".to_string(),
    };

    let result = BankService::create_bank_account(&state, user_id, req).await;

    if let Err(e) = &result {
        println!("Add bank failed: {:?}", e);
    }
    assert!(result.is_ok());

    // 5. Verify DB
    let account = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .first::<payego_primitives::models::BankAccount>(conn)
        .unwrap();

    assert_eq!(account.account_name, Some("Test User Account".to_string()));
    assert_eq!(
        account.paystack_recipient_code,
        Some("RCP_123456".to_string())
    );

    // 6. Cleanup
    diesel::delete(bank_accounts::table.filter(bank_accounts::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}
