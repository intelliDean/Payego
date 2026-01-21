use validator::ValidationError;

const MIN_LEN: usize = 12;
const MAX_LEN: usize = 128;
const SPECIAL_CHARS: &str = "!@#$%^&*";

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let len = password.len();

    if len < MIN_LEN {
        return Err(error("password_too_short"));
    }

    if len > MAX_LEN {
        return Err(error("password_too_long"));
    }

    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in password.chars() {
        match c {
            c if c.is_ascii_lowercase() => has_lower = true,
            c if c.is_ascii_uppercase() => has_upper = true,
            c if c.is_ascii_digit() => has_digit = true,
            c if SPECIAL_CHARS.contains(c) => has_special = true,
            _ => return Err(error("password_invalid_character")),
        }
    }

    // this, right here, is very important in the enforcing
    if !(has_lower && has_upper && has_digit && has_special) {
        return Err(error("password_policy_violation"));
    }

    Ok(())
}

fn error(code: &'static str) -> ValidationError {
    let mut err = ValidationError::new(code);
    err.add_param("min_length".into(), &MIN_LEN);
    err.add_param("max_length".into(), &MAX_LEN);
    err.add_param("special_chars".into(), &SPECIAL_CHARS);
    err
}
