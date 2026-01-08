use payego::services::bank_service::BankService;
use payego::models::models::AppState;
use payego::handlers::bank::BankRequest;
use diesel::{r2d2, PgConnection, Connection};
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use uuid::Uuid;
use serde_json::json;
use payego::schema::{users, bank_accounts};
use diesel::prelude::*;

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

    // 3. Setup AppState
    let state = Arc::new(AppState {
        db: pool.clone(),
        jwt_secret: "secret".to_string(),
        stripe_secret_key: "sk_test".to_string(),
        app_url: "http://localhost:8080".to_string(),
        exchange_api_url: "http://unused".to_string(),
        paypal_api_url: "http://unused".to_string(),
        paystack_api_url: base_url.clone(), 
    });

    // 4. Call Service
    let req = BankRequest {
        bank_name: "Test Bank".to_string(),
        bank_code: "057".to_string(),
        account_number: "0001234567".to_string(),
    };

    let result = BankService::add_bank_account(state, user_id, req).await;
    
    if let Err(e) = &result {
        println!("Add bank failed: {:?}", e);
    }
    assert!(result.is_ok());

    // 5. Verify DB
    let account = bank_accounts::table
        .filter(bank_accounts::user_id.eq(user_id))
        .first::<payego::models::models::BankAccount>(conn)
        .unwrap();

    assert_eq!(account.account_name, Some("Test User Account".to_string()));
    assert_eq!(account.paystack_recipient_code, Some("RCP_123456".to_string()));

    // 6. Cleanup
    diesel::delete(bank_accounts::table.filter(bank_accounts::user_id.eq(user_id))).execute(conn).unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id))).execute(conn).unwrap();
}
