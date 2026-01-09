mod common;

use payego::models::models::NewTransaction;
use uuid::Uuid;

#[test]
fn test_new_transaction_creation() {
    let user_id = Uuid::new_v4();
    let recipient_id = Some(Uuid::new_v4());
    let reference = Uuid::new_v4();

    let tx = NewTransaction {
        user_id,
        recipient_id,
        amount: 1000,
        transaction_type: "internal_transfer_send".to_string(),
        currency: "USD".to_string(),
        status: "completed".to_string(),
        provider: Some("internal".to_string()),
        description: Some("Test transfer".to_string()),
        reference,
        metadata: None,
    };

    assert_eq!(tx.user_id, user_id);
    assert_eq!(tx.recipient_id, recipient_id);
    assert_eq!(tx.amount, 1000);
    assert_eq!(tx.transaction_type, "internal_transfer_send");
    assert_eq!(tx.status, "completed");
    assert_eq!(tx.reference, reference);
}

#[test]
fn test_transaction_reference_uniqueness() {
    let user_id = Uuid::new_v4();

    let tx1 = NewTransaction {
        user_id,
        recipient_id: None,
        amount: 500,
        transaction_type: "top_up".to_string(),
        currency: "USD".to_string(),
        status: "completed".to_string(),
        provider: Some("stripe".to_string()),
        description: None,
        reference: Uuid::new_v4(),
        metadata: None,
    };

    let tx2 = NewTransaction {
        user_id,
        recipient_id: None,
        amount: 500,
        transaction_type: "top_up".to_string(),
        currency: "USD".to_string(),
        status: "completed".to_string(),
        provider: Some("stripe".to_string()),
        description: None,
        reference: Uuid::new_v4(),
        metadata: None,
    };

    assert_ne!(tx1.reference, tx2.reference);
}
