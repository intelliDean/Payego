use axum::response::{IntoResponse, Response};
use diesel::r2d2;
use http::StatusCode;
use std::fmt;
use stripe::WebhookError;
use thiserror::Error;

#[derive(Debug)]
pub enum ApiError {
    Database(diesel::result::Error),
    Argon2(argon2::password_hash::Error),
    Validation(validator::ValidationErrors),
    DatabaseConnection(String),
    Token(String),
    Auth(AuthError),
    Payment(String),
    Webhook(WebhookError),
    Internal(String),
    // PaystackError(PaystackError)
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Database(e) => write!(f, "Database error: {}", e),
            ApiError::Argon2(e) => write!(f, "Argon2 error: {}", e),
            ApiError::Validation(e) => write!(f, "Validation error: {}", e),
            ApiError::DatabaseConnection(e) => write!(f, "Database connection error: {}", e),
            ApiError::Token(e) => write!(f, "Token error: {}", e),
            ApiError::Auth(e) => write!(f, "Authentication error: {}", e),
            ApiError::Payment(e) => write!(f, "Payment error: {}", e),
            ApiError::Webhook(e) => write!(f, "Webhook error: {}", e),
            ApiError::Internal(e) => write!(f, "Internal error: {}", e),
            // ApiError::PaystackError(e) => write!(f, "Paystack error {}", e),
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApiError::Database(e) => Some(e),
            ApiError::Argon2(e) => Some(e),
            ApiError::Validation(e) => Some(e),
            ApiError::Webhook(e) => Some(e),
            _ => None,
        }
    }
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

impl From<argon2::password_hash::Error> for ApiError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ApiError::Argon2(err)
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
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => (StatusCode::BAD_REQUEST, format!("Database error: {}", e)),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                ),
            },
            ApiError::Argon2(_) => (
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
            ApiError::Auth(e) => match e {
                AuthError::MissingHeader => (
                    StatusCode::UNAUTHORIZED,
                    "Authorization header required".to_string(),
                ),
                AuthError::InvalidFormat => (
                    StatusCode::BAD_REQUEST,
                    "Invalid Authorization format".to_string(),
                ),
                AuthError::InvalidToken(msg) => {
                    (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", msg))
                }
                AuthError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid credentials".to_string(),
                ),
                AuthError::BlacklistedToken => (
                    StatusCode::UNAUTHORIZED,
                    "Token has been invalidated".to_string(),
                ),
                AuthError::InternalError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", msg),
                ),
            },
            ApiError::Payment(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Payment provider error: {}", msg),
            ),
            ApiError::Webhook(e) => match e {
                WebhookError::BadSignature | WebhookError::BadTimestamp(_) => {
                    (StatusCode::BAD_REQUEST, format!("Webhook error: {}", e))
                }
                WebhookError::BadKey => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Webhook error: invalid secret key".to_string(),
                ),
                _ => (
                    StatusCode::UNAUTHORIZED,
                    "Webhook error: Unauthorized".to_string(),
                ),
            },
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", msg),
            ),
            // ApiError::PaystackError(e) => match e {
            //     PaystackError::Configuration(msg) => (
            //
            //         ),
            //     PaystackError::RequestFailed => (),
            //     PaystackError::Api(msg) => (
            //
            //         )
            // },

        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body): (StatusCode, String) = self.into();
        (status, body).into_response()
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidFormat,
    InvalidToken(String),
    InvalidCredentials,
    BlacklistedToken,
    InternalError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "Authorization header required"),
            AuthError::InvalidFormat => write!(f, "Invalid Authorization format"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::BlacklistedToken => write!(f, "Token has been invalidated"),
            AuthError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        ApiError::Auth(err)
    }
}



#[derive(Debug, Error)]
pub enum PaystackError {
    #[error("Paystack configuration error: {0}")]
    Configuration(&'static str),

    #[error("Paystack API request failed")]
    RequestFailed,

    #[error("Paystack returned unsuccessful response: {0}")]
    Api(String),
}