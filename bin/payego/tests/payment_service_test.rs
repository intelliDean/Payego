use diesel::prelude::*;
use payego_core::services::payment_service::PaymentService;
use payego_primitives::models::top_up_dto::TopUpRequest;
use payego_primitives::schema::{transactions, users};
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

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

    // 3. Setup AppState
    let mut base_state = (*common::create_test_app_state()).clone();
    base_state.config.paypal_details.paypal_api_url = base_url.clone();
    let state = Arc::new(base_state);

    let pool = &state.db;
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

    use payego_primitives::models::entities::enum_types::{CurrencyCode, PaymentProvider};
    use std::env;

    // 4. Call Service
    let req = TopUpRequest {
        amount: 1000.0, // $10.00
        currency: CurrencyCode::USD,
        provider: PaymentProvider::Paypal,
        idempotency_key: "topup_1".to_string(),
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
    use payego_primitives::models::transaction::Transaction;
    let tx = transactions::table
        .filter(transactions::user_id.eq(user_id))
        .filter(transactions::metadata.is_not_null()) // check if it has metadata
        .first::<Transaction>(conn)
        .unwrap();

    assert_eq!(tx.amount, 100000); // 1000 * 100
    assert_eq!(
        tx.provider,
        Some(payego_primitives::models::entities::enum_types::PaymentProvider::Paypal)
    );
    assert_eq!(
        tx.txn_state,
        payego_primitives::models::entities::enum_types::PaymentState::Pending
    );

    // 6. Cleanup
    diesel::delete(transactions::table.filter(transactions::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}
