use payego::services::conversion_service::ConversionService;
use payego::models::models::{AppState, Wallet};
use payego::handlers::internal_conversion::ConvertRequest;
use diesel::{r2d2, PgConnection, Connection};
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use uuid::Uuid;
use serde_json::json;

// Helper to create a test pool
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

    // 2. Setup DB (using transaction rollback pattern if possible, but here we might just insert and hope for rollback? 
    // Ideally we use a test transaction, but the Service uses the pool directly.
    // For this "Integration/Unit" test, we'll manually cleanup or just assume a clean DB state is not guaranteed.
    // A better approach for Service testing is to let the Service accept a specific Connection, but it takes AppState (Pool).
    // We will proceed with the Pool, and insert a user/wallet first.
    
    let pool = get_test_pool();
    let conn = &mut pool.get().unwrap();
    
    // Create dummy user and wallet using Diesel (assuming simple schema)
    // We'll skip creating a user if there's no easy way, assuming we have a seed or create one:
    
    use payego::schema::{users, wallets, transactions};
    use diesel::prelude::*;
    
    let user_id = Uuid::new_v4();
    let email = format!("test_convert_{}@example.com", user_id);
    
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
            wallets::currency.eq("USD"),
        ))
        .execute(conn)
        .unwrap();

    // 3. Setup AppState
    let state = Arc::new(AppState {
        db: pool.clone(),
        jwt_secret: "secret".to_string(),
        stripe_secret_key: "sk_test".to_string(),
        app_url: "http://localhost:8080".to_string(),
        exchange_api_url,
        paypal_api_url: "http://unused".to_string(),
        paystack_api_url: "http://unused".to_string(),
    });

    // 4. Call Service
    let req = ConvertRequest {
        amount: 10.0,
        from_currency: "USD".to_string(),
        to_currency: "NGN".to_string(),
        idempotency_key: "key123".to_string(),
    };

    let result = ConversionService::convert_currency(state, user_id, req).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    
    // 5. Assertions
    // Amount: 10 * 1500 = 15000 NGN
    // Fee: 1% of 15000 = 150 NGN
    // Net: 14850 NGN = 1485000 kobo ? No, code uses cents logic.
    // Code says: amount_cents = req.amount * 100
    // fee = 0.01 * 10 * 1500 = 150.
    // fee_cents = 150 * 100 = 15000 ??
    // Wait, let's check code logic:
    // let fee = 0.01 * req.amount * exchange_rate; (0.01 * 10 * 1500 = 150.0)
    // let fee_cents = (fee * 100.0).round() as i64; (150 * 100 = 15000)
    // let result_cents = (amount * rate * 100) - fee_cents
    // (10 * 100 * 1500) ? No. 
    // Logic: 
    // amount_cents = 1000
    // converted_cents = 1000 * 1500 = 1,500,000 (Incorrect logic in Service?)
    
    // Service Logic:
    // let amount_cents = (req.amount * 100.0).round() as i64; // 1000
    // let converted_cents = ((amount_cents as f64) * exchange_rate).round() as i64; // 1000 * 1500 = 1,500,000 cents (15,000.00)
    // Note: exchange rate for USD to NGN is e.g. 1500. So 1 USD = 1500 NGN.
    // 10 USD = 15000 NGN.
    // 1000 US cents = 1,500,000 NGN kobo. Correct.
    
    // Fee logic:
    // fee = 0.01 * 10 * 1500 = 150.0 (NGN)
    // fee_cents = 150.0 * 100.0 = 15000 (kobo)
    
    // Net = 1,500,000 - 15,000 = 1,485,000 kobo (14,850.00 NGN)
    
    assert_eq!(response.converted_amount, 14850.0);
    assert_eq!(response.fee, 150.0);

    // Verify DB state
    let wallet_usd = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq("USD"))
        .first::<Wallet>(conn)
        .unwrap();
    assert_eq!(wallet_usd.balance, 0); // 10000 - 10000 (1000 used?? 10.0*100=1000. Start 10000) -> 9000
    // Wait start balance 10000 ($100). req.amount 10.0 ($10). 
    assert_eq!(wallet_usd.balance, 9000);

    let wallet_ngn = wallets::table
        .filter(wallets::user_id.eq(user_id))
        .filter(wallets::currency.eq("NGN"))
        .first::<Wallet>(conn)
        .unwrap();
    assert_eq!(wallet_ngn.balance, 1485000);
    
    // Cleanup
    diesel::delete(wallets::table.filter(wallets::user_id.eq(user_id))).execute(conn).unwrap();
    diesel::delete(transactions::table.filter(transactions::user_id.eq(user_id))).execute(conn).unwrap();
    diesel::delete(users::table.filter(users::id.eq(user_id))).execute(conn).unwrap();
}
