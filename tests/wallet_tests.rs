mod common;

use payego::handlers::transfer_internal::TransferRequest;
use validator::Validate;
use serde_json::json;

#[test]
fn test_transfer_request_validation() {
    // Valid request
    let req = serde_json::from_value::<TransferRequest>(json!({
        "amount": 100.0,
        "recipient_email": "test@example.com",
        "currency": "USD"
    })).unwrap();
    assert!(req.validate().is_ok());

    // Invalid amount (too low)
    let req = serde_json::from_value::<TransferRequest>(json!({
        "amount": 0.5,
        "recipient_email": "test@example.com",
        "currency": "USD"
    })).unwrap();
    assert!(req.validate().is_err());

    // Invalid amount (too high)
    let req = serde_json::from_value::<TransferRequest>(json!({
        "amount": 20000.0,
        "recipient_email": "test@example.com",
        "currency": "USD"
    })).unwrap();
    assert!(req.validate().is_err());

    // Invalid email
    let req = serde_json::from_value::<TransferRequest>(json!({
        "amount": 100.0,
        "recipient_email": "not-an-email",
        "currency": "USD"
    })).unwrap();
    assert!(req.validate().is_err());

    // Invalid currency
    let req = serde_json::from_value::<TransferRequest>(json!({
        "amount": 100.0,
        "recipient_email": "test@example.com",
        "currency": "UNKNOWN"
    })).unwrap();
    assert!(req.validate().is_err());
}

#[test]
fn test_amount_to_cents_conversion() {
    // This is essentially testing the logic inside the handler
    // Since it's not a separate function yet, we'll just verify the math logic
    let amount: f64 = 10.99;
    let cents = (amount * 100.0).round() as i64;
    assert_eq!(cents, 1099);

    let amount: f64 = 0.01;
    let cents = (amount * 100.0).round() as i64;
    assert_eq!(cents, 1);

    let amount: f64 = 100.0;
    let cents = (amount * 100.0).round() as i64;
    assert_eq!(cents, 10000);
}
