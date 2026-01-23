use diesel::prelude::*;
use payego_core::services::conversion_service::ConversionService;
use payego_primitives::models::conversion_dto::ConvertRequest;
use payego_primitives::models::entities::enum_types::CurrencyCode;
use payego_primitives::models::entities::wallet::Wallet;
use payego_primitives::schema::{users, wallets};
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

#[tokio::test]
#[serial]
async fn test_convert_currency_success() {
    // 1. Setup WireMock
    let mock_server = MockServer::start().await;
    let exchange_api_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/USD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rates": {
                "NGN": 1500.0
            }
        })))
        .mount(&mock_server)
        .await;

    // 2. Setup AppState
    let mut base_state = (*common::create_test_app_state()).clone();
    base_state.config.exchange_api_url = exchange_api_url;
    let state = Arc::new(base_state);

    let pool = &state.db;
    let conn = &mut pool.get().unwrap();

    // 3. Setup Data
    let user_id = Uuid::new_v4();
    let email = format!("test_convert_success_{}@example.com", user_id);

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::email.eq(email),
            users::password_hash.eq("hash"),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(wallets::table)
        .values((
            wallets::id.eq(Uuid::new_v4()),
            wallets::user_id.eq(user_id),
            wallets::balance.eq(10000), // $100.00
            wallets::currency.eq(CurrencyCode::USD),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(wallets::table)
        .values((
            wallets::id.eq(Uuid::new_v4()),
            wallets::user_id.eq(user_id),
            wallets::balance.eq(0),
            wallets::currency.eq(CurrencyCode::NGN),
        ))
        .execute(conn)
        .unwrap();

    // 4. Call Service
    let req = ConvertRequest {
        amount_cents: 1000,
        from_currency: CurrencyCode::USD,
        to_currency: CurrencyCode::NGN,
        idempotency_key: format!("key_{}", Uuid::new_v4()),
    };

    let result = ConversionService::convert_currency(&state, user_id, req).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // 5. Assertions
    // Amount: 10 * 1500 = 15000 NGN
    // Fee: 1% = 150 NGN
    // Net: 14850 NGN

    assert_eq!(response.converted_amount, 14850.0);
    assert_eq!(response.fee, 150.0);

    // Verify DB state
    let wallet_usd = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(CurrencyCode::USD))
        .first::<Wallet>(conn)
        .unwrap();
    // 10000 - 1000 = 9000
    assert_eq!(wallet_usd.balance, 9000);

    let wallet_ngn = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(CurrencyCode::NGN))
        .first::<Wallet>(conn)
        .unwrap();
    // 14850 * 100 = 1485000
    assert_eq!(wallet_ngn.balance, 1485000);

    // Cleanup
    diesel::delete(wallets::table.filter(wallets::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}

#[tokio::test]
#[serial]
async fn test_convert_currency_insufficient_balance() {
    // 1. Setup WireMock
    let mock_server = MockServer::start().await;
    let exchange_api_url = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/USD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rates": {
                "NGN": 1500.0
            }
        })))
        .mount(&mock_server)
        .await;

    // 2. Setup AppState
    let mut base_state = (*common::create_test_app_state()).clone();
    base_state.config.exchange_api_url = exchange_api_url;
    let state = Arc::new(base_state);

    let pool = &state.db;
    let conn = &mut pool.get().unwrap();

    // 3. Setup Data
    let user_id = Uuid::new_v4();
    let email = format!("test_convert_fail_{}@example.com", user_id);

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::email.eq(email),
            users::password_hash.eq("hash"),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(wallets::table)
        .values((
            wallets::id.eq(Uuid::new_v4()),
            wallets::user_id.eq(user_id),
            wallets::balance.eq(500), // $5.00
            wallets::currency.eq(CurrencyCode::USD),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(wallets::table)
        .values((
            wallets::id.eq(Uuid::new_v4()),
            wallets::user_id.eq(user_id),
            wallets::balance.eq(0),
            wallets::currency.eq(CurrencyCode::NGN),
        ))
        .execute(conn)
        .unwrap();

    // 4. Call Service with amount > balance
    let req = ConvertRequest {
        amount_cents: 1000, // $10.00
        from_currency: CurrencyCode::USD,
        to_currency: CurrencyCode::NGN,
        idempotency_key: format!("key_{}", Uuid::new_v4()),
    };

    let result = ConversionService::convert_currency(&state, user_id, req).await;

    assert!(result.is_err());
    // Only check debug string for now since ApiError doesn't easy partial match
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Insufficient balance") || err_msg.contains("Payment error"));

    // Cleanup
    diesel::delete(wallets::table.filter(wallets::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}

#[tokio::test]
async fn test_convert_currency_same_currency() {
    let state = common::create_test_app_state();
    let user_id = Uuid::new_v4();

    let req = ConvertRequest {
        amount_cents: 1000,
        from_currency: CurrencyCode::USD,
        to_currency: CurrencyCode::USD,
        idempotency_key: "any".to_string(),
    };

    let result = ConversionService::convert_currency(&state, user_id, req).await;
    assert!(result.is_err());
}
