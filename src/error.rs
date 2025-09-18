use diesel::r2d2;
use http::StatusCode;
use stripe::WebhookError;

#[derive(Debug)]
pub enum ApiError {
    Database(diesel::result::Error),
    Bcrypt(bcrypt::BcryptError),
    Validation(validator::ValidationErrors),
    DatabaseConnection(String),
    Token(String),
    Auth(String),
    Payment(String),
    Webhook(WebhookError),
}

impl From<r2d2::Error> for ApiError {
    fn from(err: r2d2::Error) -> Self {
        ApiError::DatabaseConnection(err.to_string())
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(err: diesel::result::Error) -> Self {
        ApiError::Database(err)
    }
}

impl From<bcrypt::BcryptError> for ApiError {
    fn from(err: bcrypt::BcryptError) -> Self {
        ApiError::Bcrypt(err)
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::Token(err)
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::Validation(err)
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::Payment(err.to_string())
    }
}

impl From<stripe::WebhookError> for ApiError {
    fn from(err: stripe::WebhookError) -> Self {
        ApiError::Webhook(err)
    }
}

impl From<ApiError> for (StatusCode, String) {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::Database(e) => match e {
                diesel::result::Error::NotFound => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid email or password".to_string(),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                ),
            },
            ApiError::Bcrypt(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password verification error".to_string(),
            ),
            ApiError::Validation(errors) => (
                StatusCode::BAD_REQUEST,
                format!("Validation error: {}", errors),
            ),
            ApiError::DatabaseConnection(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database connection error: {}", e),
            ),
            ApiError::Token(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token creation error: {}", e),
            ),
            ApiError::Auth(msg) => (
                StatusCode::UNAUTHORIZED,
                format!("Auth error: {}", msg),
            ),
            ApiError::Payment(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Payment provider error: {}", msg),
            ),
            ApiError::Webhook(e) => match e {
                WebhookError::BadSignature | WebhookError::BadTimestamp(_) => (
                    StatusCode::BAD_REQUEST,
                    format!("Webhook error: {}", e),
                ),
                WebhookError::BadKey => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Webhook error: invalid secret key".to_string(),
                ),
                _ => {(StatusCode::UNAUTHORIZED, "Webhook error: Unauthorized".to_string())}
            },
        }
    }
}