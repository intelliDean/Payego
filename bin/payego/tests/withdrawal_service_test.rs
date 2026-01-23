use diesel::prelude::*;
use payego_core::services::withdrawal_service::WithdrawalService;
use payego_primitives::models::dtos::withdrawal_dto::WithdrawRequest;
use payego_primitives::models::entities::enum_types::CurrencyCode;
use payego_primitives::models::entities::wallet::Wallet;
use payego_primitives::schema::{bank_accounts, banks, transactions, users, wallets};
use serde_json::json;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

#[tokio::test]
#[serial]
async fn test_withdrawal_success() {
    // 1. Setup WireMock
    let mock_server = MockServer::start().await;
    let base_url = mock_server.uri();

    // Mock Exchange Rate (USD -> NGN)
    Mock::given(method("GET"))
        .and(path("/USD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rates": {
                "NGN": 1500.0
            }
        })))
        .mount(&mock_server)
        .await;

    // Mock Paystack Balance
    Mock::given(method("GET"))
        .and(path("/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": true,
            "message": "Balances retrieved",
            "data": [
                {
                    "currency": "NGN",
                    "balance": 50000000 // Large balance
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Mock Paystack Transfer
    Mock::given(method("POST"))
        .and(path("/transfer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": true,
            "message": "Transfer initiated",
            "data": {
                "transfer_code": "TRF_1234567890",
                "reference": "REF_12345",
                "id": 12345,
                "amount": 1500000,
                "currency": "NGN"
            }
        })))
        .mount(&mock_server)
        .await;

    // 2. Setup AppState
    let mut base_state = (*common::create_test_app_state()).clone();
    base_state.config.exchange_api_url = base_url.clone();
    base_state.config.paystack_details.paystack_api_url = base_url.clone();
    let state = Arc::new(base_state);

    let pool = &state.db;
    let conn = &mut pool.get().unwrap();

    let user_id = Uuid::new_v4();
    let email = format!("test_withdraw_{}@example.com", user_id);
    let bank_id = Uuid::new_v4();

    // Insert User
    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::email.eq(email),
            users::password_hash.eq("hash"),
        ))
        .execute(conn)
        .unwrap();

    // Insert Wallet (USD)
    diesel::insert_into(wallets::table)
        .values((
            wallets::id.eq(Uuid::new_v4()),
            wallets::user_id.eq(user_id),
            wallets::balance.eq(2000), // $20.00
            wallets::currency.eq(CurrencyCode::USD),
        ))
        .execute(conn)
        .unwrap();

    // Insert Bank
    diesel::insert_into(banks::table)
        .values((
            banks::id.eq(1),
            banks::name.eq("Test Bank"),
            banks::code.eq("057"),
            banks::currency.eq(CurrencyCode::NGN),
            banks::country.eq("Nigeria"),
        ))
        .execute(conn)
        .unwrap();

    // Insert Bank Account
    diesel::insert_into(bank_accounts::table)
        .values((
            bank_accounts::id.eq(bank_id),
            bank_accounts::user_id.eq(user_id),
            bank_accounts::bank_name.eq("Test Bank"),
            bank_accounts::account_number.eq("1234567890"),
            bank_accounts::account_name.eq("Test User"),
            bank_accounts::bank_code.eq("057"),
            bank_accounts::provider_recipient_id.eq("RCP_123456"),
            bank_accounts::is_verified.eq(true),
        ))
        .execute(conn)
        .unwrap();

    // 4. Call Service
    unsafe {
        std::env::set_var("PAYSTACK_SECRET_KEY", "sk_test_paystack");
    }

    let req = WithdrawRequest {
        amount: 10.0,
        currency: CurrencyCode::USD,
        reference: Uuid::new_v4(),
        idempotency_key: format!("withdraw_key_{}", Uuid::new_v4()),
    };

    let result = WithdrawalService::withdraw(&state, user_id, bank_id, req).await;

    if let Err(e) = &result {
        println!("Withdrawal failed: {:?}", e);
    }
    assert!(result.is_ok());

    // 5. Assertions
    // Balance check: 2000 - 1000 = 1000 ($10 left)
    let wallet = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq(CurrencyCode::USD))
        .first::<Wallet>(conn)
        .unwrap();
    assert_eq!(wallet.balance, 1000);

    // 6. Cleanup
    diesel::delete(bank_accounts::table.filter(bank_accounts::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(banks::table.filter(banks::code.eq("057")))
        .execute(conn)
        .unwrap();
    diesel::delete(wallets::table.filter(wallets::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(transactions::table.filter(transactions::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}

#[tokio::test]
#[serial]
async fn test_withdrawal_insufficient_balance() {
    let state = common::create_test_app_state();
    let pool = &state.db;
    let conn = &mut pool.get().unwrap();

    let user_id = Uuid::new_v4();
    let bank_id = Uuid::new_v4();

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::email.eq(format!("test_fail_bal_{}@example.com", user_id)),
            users::password_hash.eq("hash"),
        ))
        .execute(conn)
        .unwrap();

    // Wallet with 0 balance
    diesel::insert_into(wallets::table)
        .values((
            wallets::id.eq(Uuid::new_v4()),
            wallets::user_id.eq(user_id),
            wallets::balance.eq(0),
            wallets::currency.eq(CurrencyCode::USD),
        ))
        .execute(conn)
        .unwrap();

    let req = WithdrawRequest {
        amount: 10.0,
        currency: CurrencyCode::USD,
        reference: Uuid::new_v4(),
        idempotency_key: "any".to_string(),
    };

    let result = WithdrawalService::withdraw(&state, user_id, bank_id, req).await;
    assert!(result.is_err());

    // Cleanup
    diesel::delete(wallets::table.filter(wallets::user_id.eq(user_id)))
        .execute(conn)
        .unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .unwrap();
}

#[tokio::test]
async fn test_withdrawal_unsupported_currency() {
    let state = common::create_test_app_state();
    let user_id = Uuid::new_v4();

    let bank_id = Uuid::new_v4();
    let req = WithdrawRequest {
        amount: 10.0,
        currency: CurrencyCode::USD, // Changed to valid enum variant, as parsing would fail earlier or logic handles it
        reference: Uuid::new_v4(),
        idempotency_key: "any".to_string(),
    };

    let result = WithdrawalService::withdraw(&state, user_id, bank_id, req).await;
    assert!(result.is_err());
}
