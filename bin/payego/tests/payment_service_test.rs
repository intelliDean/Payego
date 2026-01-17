use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, Connection, PgConnection};
use dotenvy::dotenv;
use payego_core::services::payment_service::PaymentService;
use payego_primitives::models::{AppState, TopUpRequest, TopUpResponse, Wallet};
use payego_primitives::schema::{transactions, users};
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
async fn test_top_up_paypal_init_success() {
    // 1. Setup WireMock
    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    // Mock PayPal Token
    Mock::given(method("POST"))
        .and(path("/v1/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "ACCESS_TOKEN_XYZ",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // Mock PayPal Order Creation
    Mock::given(method("POST"))
        .and(path("/v2/checkout/orders"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "ORDER_123",
            "links": [
                {
                    "href": "https://approval.url",
                    "rel": "approve",
                    "method": "GET"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // 2. Setup DB
    let pool = get_test_pool();
    let conn = &mut pool.get().unwrap();

    let user_id = Uuid::new_v4();
    let email = format!("test_topup_{}@example.com", user_id);

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
        paypal_api_url: base_url.clone(),
        stripe_api_url: "http://unused".to_string(),
        stripe_webhook_secret: Default::default(),
        paystack_api_url: "http://unused".to_string(),
        paystack_secret_key: SecretString::from("sk_test_paystack"),
        paystack_webhook_secret: Default::default(),
        paypal_client_id: "test_client_id".to_string(),
        paypal_secret: SecretString::from("test_secret"),
    });

    // 4. Call Service
    let req = TopUpRequest {
        amount: 50.0, // $50.00
        currency: "USD".to_string(),
        provider: "paypal".to_string(),
        idempotency_key: "topup_key_1".to_string(),
        reference: Uuid::new_v4(),
    };

    // Need to set PAYPAL vars for test
    unsafe {
        env::set_var("PAYPAL_CLIENT_ID", "test_client");
        env::set_var("PAYPAL_SECRET", "test_secret");
    }

    let result = PaymentService::initiate_top_up(&state, user_id, req).await;

    if let Err(e) = &result {
        println!("TopUp failed: {:?}", e);
    }
    assert!(result.is_ok());
    let response = result.unwrap();

    assert!(response.session_url.is_some());
    assert_eq!(response.session_url.unwrap(), "https://approval.url");

    // 5. Verify Transaction
    // The service inserts a transaction with "pending" status.
    use payego_primitives::models::Transaction;
    let tx = transactions::table
        .filter(transactions::user_id.eq(user_id))
        .filter(transactions::metadata.is_not_null()) // check if it has metadata
        .first::<Transaction>(conn)
        .unwrap();

    assert_eq!(tx.amount, 5000); // 50 * 100
    assert_eq!(tx.provider, Some("paypal".to_string()));
    assert_eq!(tx.status, "pending");

    // 6. Cleanup
    diesel::delete(transactions::table.filter(transactions::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}
