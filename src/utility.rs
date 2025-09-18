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