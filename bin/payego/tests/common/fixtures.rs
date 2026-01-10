use fake::{Fake, Faker};
use payego_primitives::models::RegisterRequest;
use uuid::Uuid;

/// Create a test user registration request with random data
pub fn create_test_register_request() -> RegisterRequest {
    RegisterRequest {
        email: format!("test{}@example.com", Uuid::new_v4()),
        password: "SecurePass123!".to_string(),
        username: Some(format!("user{}", Uuid::new_v4())),
    }
}

/// Create a test user with specific email
pub fn create_test_register_request_with_email(email: &str) -> RegisterRequest {
    RegisterRequest {
        email: email.to_string(),
        password: "SecurePass123!".to_string(),
        username: Some(format!("user{}", Uuid::new_v4())),
    }
}
