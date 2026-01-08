use payego::services::withdrawal_service::WithdrawalService;
use payego::models::models::{AppState, Wallet, BankAccount};
use payego::handlers::withdraw::WithdrawRequest;
use diesel::{r2d2, PgConnection, Connection};
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use uuid::Uuid;
use serde_json::json;
use payego::schema::{users, wallets, transactions, bank_accounts};
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
                "id": 12345,
                "amount": 1500000,
                "currency": "NGN"
            }
        })))
        .mount(&mock_server)
        .await;


    // 2. Setup DB
    let pool = get_test_pool();
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
            wallets::currency.eq("USD"),
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
            bank_accounts::paystack_recipient_code.eq("RCP_123456"),
        ))
        .execute(conn)
        .unwrap();


    // 3. Setup AppState
    let state = Arc::new(AppState {
        db: pool.clone(),
        jwt_secret: "secret".to_string(),
        stripe_secret_key: "sk_test".to_string(),
        app_url: "http://localhost:8080".to_string(),
        exchange_api_url: base_url.clone(), // Use same mock server
        paypal_api_url: "http://unused".to_string(),
        paystack_api_url: base_url.clone(), // Use same mock server for simplicity, paths are distinct
    });

    // 4. Call Service
    let req = WithdrawRequest {
        amount: 10.0,
        currency: "USD".to_string(),
        bank_id: bank_id.to_string(),
        reference: Uuid::new_v4(),
        idempotency_key: "withdraw_key_1".to_string(),
    };

    let result = WithdrawalService::initiate_withdrawal(state, user_id, req).await;
    
    if let Err(e) = &result {
        println!("Withdrawal failed: {:?}", e);
    }
    assert!(result.is_ok());

    // 5. Assertions
    // Balance check: 2000 - 1000 = 1000 ($10 left)
    let wallet = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq("USD"))
        .first::<Wallet>(conn)
        .unwrap();
    assert_eq!(wallet.balance, 1000);

    // 6. Cleanup
    diesel::delete(bank_accounts::table.filter(bank_accounts::user_id.eq(user_id))).execute(conn).unwrap();
    diesel::delete(wallets::table.filter(wallets::user_id.eq(user_id))).execute(conn).unwrap();
    diesel::delete(transactions::table.filter(transactions::user_id.eq(user_id))).execute(conn).unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id))).execute(conn).unwrap();
}
