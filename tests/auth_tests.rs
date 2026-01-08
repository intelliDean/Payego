mod common;

use payego::config::security_config::{create_token, verify_token};
use common::create_test_app_state;

#[tokio::test]
async fn test_create_and_verify_token() {
    let state = create_test_app_state();
    let user_id = "test-user-123";
    
    // Create token
    let token = create_token(&state, user_id).expect("Failed to create token");
    
    // Verify it's not empty
    assert!(!token.is_empty());
    
    // Verify token
    let claims = verify_token(&state, &token).expect("Failed to verify token");
    
    // Check claims
    assert_eq!(claims.sub, user_id);
    assert!(claims.exp > claims.iat);
}

#[tokio::test]
async fn test_invalid_token_rejected() {
    let state = create_test_app_state();
    
    // Try to verify an invalid token
    let result = verify_token(&state, "invalid.token.here");
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_with_wrong_secret_rejected() {
    let state = create_test_app_state();
    let user_id = "test-user-456";
    
    // Create token with correct secret
    let token = create_token(&state, user_id).expect("Failed to create token");
    
    // Create a different state with different secret
    let mut different_state = (*state).clone();
    different_state.jwt_secret = "different_secret_key_minimum_32_characters_long".to_string();
    
    // Try to verify with wrong secret
    let result = verify_token(&std::sync::Arc::new(different_state), &token);
    
    assert!(result.is_err());
}

#[test]
fn test_password_hashing() {
    let password = "SecurePassword123!";
    let hash = bcrypt::hash(password, 12).unwrap();
    
    // Correct password should verify
    assert!(bcrypt::verify(password, &hash).unwrap());
    
    // Wrong password should not verify
    assert!(!bcrypt::verify("WrongPassword", &hash).unwrap());
}

#[test]
fn test_password_validation() {
    use payego::utility::validate_password;
    
    // Valid passwords
    assert!(validate_password("SecurePass123!").is_ok());
    assert!(validate_password("AnotherGood1!").is_ok());
    
    // Invalid passwords - too short
    assert!(validate_password("Short1!").is_err());
    
    // Invalid passwords - no numbers
    assert!(validate_password("NoNumbers!").is_err());
    
    // Invalid passwords - no special characters
    assert!(validate_password("NoSpecial123").is_err());
}
