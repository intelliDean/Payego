mod common;

use common::create_test_app_state;
use payego_primitives::config::security_config::{create_token, verify_token};
use serial_test::serial;

#[tokio::test]
#[serial]
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
    use secrecy::SecretString;
    different_state.jwt_secret =
        SecretString::from("different_secret_key_minimum_32_characters_long");

    // Try to verify with wrong secret
    let result = verify_token(&std::sync::Arc::new(different_state), &token);

    assert!(result.is_err());
}

#[test]
fn test_password_hashing() {
    use argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    };

    let password = "SecurePassword123!";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // Hash password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Parse hash
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();

    // Correct password should verify
    assert!(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok());

    // Wrong password should not verify
    assert!(argon2
        .verify_password("WrongPassword".as_bytes(), &parsed_hash)
        .is_err());
}

#[test]
fn test_password_validation() {
    use payego_primitives::utility::validate_password;

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
