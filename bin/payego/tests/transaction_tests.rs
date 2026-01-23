mod common;

use payego_primitives::models::entities::enum_types::{
    CurrencyCode, PaymentProvider, PaymentState, TransactionIntent,
};
use payego_primitives::models::entities::transaction::NewTransaction;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_new_transaction_creation() {
    let user_id = Uuid::new_v4();
    let recipient_id = Some(Uuid::new_v4());
    let reference = Uuid::new_v4();

    let tx = NewTransaction {
        user_id,
        counterparty_id: recipient_id,
        amount: 1000,
        intent: TransactionIntent::Transfer,
        currency: CurrencyCode::USD,
        txn_state: PaymentState::Completed,
        provider: Some(PaymentProvider::Internal),
        provider_reference: None,
        idempotency_key: "idemp_test",
        reference,
        description: Some("Test transfer"),
        metadata: json!({}),
    };

    assert_eq!(tx.user_id, user_id);
    assert_eq!(tx.counterparty_id, recipient_id);
    assert_eq!(tx.amount, 1000);
    assert!(matches!(tx.intent, TransactionIntent::Transfer));
    assert!(matches!(tx.txn_state, PaymentState::Completed));
    assert_eq!(tx.reference, reference);
}

#[test]
fn test_transaction_reference_uniqueness() {
    let user_id = Uuid::new_v4();

    let tx1 = NewTransaction {
        user_id,
        counterparty_id: None,
        amount: 500,
        intent: TransactionIntent::TopUp,
        currency: CurrencyCode::USD,
        txn_state: PaymentState::Pending,
        provider: Some(PaymentProvider::Stripe),
        provider_reference: Some("stripe_ref_123"),
        idempotency_key: "idemp_1",
        reference: Uuid::new_v4(),
        description: Some("Test transaction"),
        metadata: json!({}),
    };

    let tx2 = NewTransaction {
        user_id,
        counterparty_id: None,
        amount: 500,
        intent: TransactionIntent::TopUp,
        currency: CurrencyCode::USD,
        txn_state: PaymentState::Pending,
        provider: Some(PaymentProvider::Stripe),
        provider_reference: None,
        idempotency_key: "idemp_2",
        reference: Uuid::new_v4(),
        description: None,
        metadata: json!({}),
    };

    assert_ne!(tx1.reference, tx2.reference);
}
