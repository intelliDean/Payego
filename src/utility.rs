use std::sync::Mutex;
use crate::error::ApiError;
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::Client;
use tracing::log::error;
use validator::ValidationError;

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let trimmed = password.trim();

    // Check for empty or too short password
    if trimmed.is_empty() || trimmed.len() < 8 {
        return Err(ValidationError::new(
            "Password cannot be empty and must be at least 8 characters long",
        ));
    }

    // Individual character checks using iterator methods
    let mut has_lowercase = false;
    let mut has_uppercase = false;
    let mut has_digit = false;
    let mut has_special = false;
    let mut has_invalid = false;

    for c in trimmed.chars() {
        if c.is_ascii_lowercase() {
            has_lowercase = true;
        } else if c.is_ascii_uppercase() {
            has_uppercase = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if "!@#$%^&*".contains(c) {
            has_special = true;
        } else {
            has_invalid = true;
        }
    }

    if !(has_lowercase && has_uppercase && has_digit && has_special) {
        return Err(ValidationError::new(
            "Password must be at least 8 characters long and contain at \
                least one uppercase letter, one lowercase letter, one digit, \
                and one special character (!@#$%^&*)",
        ));
    }

    if has_invalid {
        return Err(ValidationError::new(
            "Password contains invalid characters. Only letters, \
                numbers, and !@#$%^&* are allowed",
        ));
    }

    Ok(())
}

// // Add this to your AppState
// pub struct AppState {
//     pub paypal_client_id: String,
//     pub paypal_secret: String,
//     pub paypal_access_token: Mutex<String>, // Use a mutex for thread safety
//     pub paypal_token_expiry: Mutex<Option<SystemTime>>,
//     // ... other fields
// }
pub async fn get_paypal_access_token(client: &Client, client_id: &str, secret: &str) -> Result<String, ApiError> {
    let resp = client
        .post("https://api-m.sandbox.paypal.com/v1/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!(
            "Basic {}",
            general_purpose::STANDARD.encode(format!("{}:{}", client_id, secret))
        ))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await
        .map_err(|e| {
            error!("PayPal token request failed: {}", e);
            ApiError::Payment(format!("Failed to fetch PayPal access token: {}", e))
        })?;

    let status = resp.status();

    let json = resp.json::<serde_json::Value>().await.map_err(|e| {
        error!("PayPal token response parsing failed: {}", e);
        ApiError::Payment(format!("Failed to parse PayPal token response: {}", e))
    })?;

    if !status.is_success() {
        error!("PayPal token API error: status {}, response {:?}", status, json);
        return Err(ApiError::Payment(format!(
            "PayPal token API error: {}",
            json["error_description"].as_str().unwrap_or("Unknown error")
        )));
    }

    json["access_token"]
        .as_str()
        .ok_or_else(|| {
            error!("Invalid PayPal token response: missing access_token");
            ApiError::Payment("Invalid PayPal token response".to_string())
        })
        .map(|s| s.to_string())
}