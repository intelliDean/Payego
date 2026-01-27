mod common;

use axum_test::TestServer;
use common::{create_test_app, create_test_app_state};
use diesel::prelude::*;
use http::StatusCode;
use payego_primitives::models::entities::enum_types::CurrencyCode;

use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[serial]
async fn test_top_up_idempotency() {
    let state = create_test_app_state();
    let _reference = Uuid::new_v4();

    // Setup WireMock for PayPal
    let mock_server = MockServer::start().await;
    let paypal_url = mock_server.uri();

    // PayPal OAuth Mock
    Mock::given(method("POST"))
        .and(path("/v1/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "mock_paypal_token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // PayPal Order Mock
    Mock::given(method("POST"))
        .and(path("/v2/checkout/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "PAY-123",
            "status": "CREATED",
            "links": [
                {
                    "href": "https://www.paypal.com/checkoutnow?token=PAY-123",
                    "rel": "approve",
                    "method": "GET"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let mut base_state = (*state).clone();
    base_state.config.paypal_details.paypal_api_url = paypal_url.clone();
    base_state.config.paypal_details.paypal_client_id = "test_client".to_string();
    base_state.config.paypal_details.paypal_secret =
        secrecy::SecretString::new("test_secret".to_string().into());
    let state = Arc::new(base_state);

    // Run migrations and cleanup
    {
        let mut conn = state.db.get().expect("Failed to get DB connection");
        common::run_test_migrations(&mut conn);
        common::cleanup_test_db(&mut conn);
    }

    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();
    let (auth_token, _) = common::create_test_user(&server, "test_topup@example.com").await;

    let _reference = Uuid::new_v4();
    let top_up_data = json!({
        "amount": 100.0,
        "provider": "Paypal",
        "currency": "USD",
        "idempotency_key": "topup_idemp_1"
    });

    // First request
    let response1 = server
        .post("/api/wallet/top_up")
        .add_header("Authorization", format!("Bearer {}", auth_token))
        .json(&top_up_data)
        .await;

    response1.assert_status(StatusCode::OK);
    let body1: serde_json::Value = response1.json();
    let tx_id1 = body1["transaction_id"].as_str().unwrap();
    // No assertion against reference here as TopUpRequest doesn't take a reference

    // Second request (idempotent)
    let response2 = server
        .post("/api/wallet/top_up")
        .add_header("Authorization", format!("Bearer {}", auth_token))
        .json(&top_up_data)
        .await;

    response2.assert_status(StatusCode::OK);
    let body2: serde_json::Value = response2.json();
    let tx_id2 = body2["transaction_id"].as_str().unwrap();
    assert_eq!(tx_id1, tx_id2);

    // Verify only one transaction exists in DB
    let mut conn = state.db.get().expect("Failed to get DB connection");
    let count = payego_primitives::schema::transactions::table
        .filter(
            payego_primitives::schema::transactions::reference.eq(Uuid::parse_str(tx_id1).unwrap()),
        )
        .count()
        .get_result::<i64>(&mut conn)
        .unwrap();

    assert_eq!(count, 1);
}

#[tokio::test]
#[serial]
async fn test_internal_transfer_idempotency() {
    let state = create_test_app_state();

    // Run migrations and cleanup
    {
        let mut conn = state.db.get().expect("Failed to get DB connection");
        common::run_test_migrations(&mut conn);
        common::cleanup_test_db(&mut conn);
    }

    let app = create_test_app(state.clone());
    let server = TestServer::new(app).unwrap();

    let sender_email = "sender@example.com";
    let (sender_token, _) = common::create_test_user(&server, sender_email).await;

    // Create recipient
    let recipient_email = "recipient@example.com";
    let recipient_id;
    {
        let mut conn = state.db.get().expect("Failed to get DB connection");

        // Use Diesel to create recipient properly
        diesel::insert_into(payego_primitives::schema::users::table)
            .values((
                payego_primitives::schema::users::email.eq(recipient_email),
                payego_primitives::schema::users::password_hash.eq("hashed"),
                payego_primitives::schema::users::username.eq("recipient"),
            ))
            .execute(&mut conn)
            .unwrap();

        recipient_id = payego_primitives::schema::users::table
            .filter(payego_primitives::schema::users::email.eq(recipient_email))
            .select(payego_primitives::schema::users::id)
            .first::<Uuid>(&mut conn)
            .unwrap();

        // Setup wallet for recipient (needed for internal transfer to work)
        diesel::insert_into(payego_primitives::schema::wallets::table)
            .values((
                payego_primitives::schema::wallets::id.eq(Uuid::new_v4()),
                payego_primitives::schema::wallets::user_id.eq(recipient_id),
                payego_primitives::schema::wallets::balance.eq(0),
                payego_primitives::schema::wallets::currency.eq(CurrencyCode::USD),
            ))
            .execute(&mut conn)
            .unwrap();

        // Setup initial balance for sender
        use payego_primitives::schema::users;
        let sender_id = users::table
            .filter(users::email.eq(sender_email))
            .select(users::id)
            .first::<Uuid>(&mut conn)
            .unwrap();

        // insert sender wallet (register doesn't create it)
        diesel::insert_into(payego_primitives::schema::wallets::table)
            .values((
                payego_primitives::schema::wallets::id.eq(Uuid::new_v4()),
                payego_primitives::schema::wallets::user_id.eq(sender_id),
                payego_primitives::schema::wallets::balance.eq(10000), // $100
                payego_primitives::schema::wallets::currency.eq(CurrencyCode::USD),
            ))
            .execute(&mut conn)
            .unwrap();
    }

    let reference = Uuid::new_v4();
    let transfer_data = json!({
        "amount": 10.0,
        "recipient": recipient_id,
        "currency": "USD",
        "description": "Internal transfer test",
        "reference": reference,
        "idempotency_key": "transfer_idemp_1"
    });

    // First request
    let response1 = server
        .post("/api/transfer/internal")
        .add_header("Authorization", format!("Bearer {}", sender_token))
        .json(&transfer_data)
        .await;

    response1.assert_status(StatusCode::OK);

    // Second request (idempotent)
    let response2 = server
        .post("/api/transfer/internal")
        .add_header("Authorization", format!("Bearer {}", sender_token))
        .json(&transfer_data)
        .await;

    response2.assert_status(StatusCode::OK);

    // Verify only one debit exists for the sender with this reference
    let mut conn = state.db.get().expect("Failed to get DB connection");

    use payego_primitives::schema::users;
    let sender_id = users::table
        .filter(users::email.eq(sender_email))
        .select(users::id)
        .first::<Uuid>(&mut conn)
        .unwrap();

    let count = payego_primitives::schema::transactions::table
        .filter(payego_primitives::schema::transactions::reference.eq(reference))
        .filter(payego_primitives::schema::transactions::user_id.eq(sender_id))
        .count()
        .get_result::<i64>(&mut conn)
        .unwrap();

    assert_eq!(count, 1);

    // Check balance - should only have deducted once ($100 - $10 = $90)
    let balance: i64 = payego_primitives::schema::wallets::table
        .filter(payego_primitives::schema::wallets::user_id.eq(sender_id))
        .filter(payego_primitives::schema::wallets::currency.eq(CurrencyCode::USD))
        .select(payego_primitives::schema::wallets::balance)
        .first::<i64>(&mut conn)
        .unwrap();

    assert_eq!(balance, 9000);
}
