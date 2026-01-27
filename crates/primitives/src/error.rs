use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::r2d2;
use http::StatusCode;
use serde::Serialize;
use serde_json::json;
use std::fmt;
use stripe::WebhookError;
use thiserror::Error;
use utoipa::ToSchema;

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
    BadRequest(String),
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
            ApiError::BadRequest(e) => write!(f, "Bad request: {}", e),
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
                AuthError::InvalidCredentials => {
                    (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
                }
                AuthError::BlacklistedToken => (
                    StatusCode::UNAUTHORIZED,
                    "Token has been invalidated".to_string(),
                ),
                AuthError::InternalError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", msg),
                ),
                AuthError::DuplicateEmail => {
                    (StatusCode::BAD_REQUEST, "Email already exist".to_string())
                }
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
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                format!("Bad request: {}", msg),
            ),
        }
    }
}

#[derive(ToSchema, Serialize)]
pub struct ApiErrorResponse {
    #[schema(example = "INVALID_CREDENTIALS")]
    pub code: String,
    #[schema(example = "Email or password is incorrect")]
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            // ── Authentication related ──
            ApiError::Auth(AuthError::MissingHeader) => (
                StatusCode::UNAUTHORIZED,
                ApiErrorResponse {
                    code: "MISSING_AUTH_HEADER".to_string(),
                    message: "Authorization header is required".to_string(),
                    details: None,
                },
            ),

            ApiError::Auth(AuthError::InvalidFormat) => (
                StatusCode::BAD_REQUEST,
                ApiErrorResponse {
                    code: "INVALID_AUTH_FORMAT".to_string(),
                    message: "Invalid Authorization header format".to_string(),
                    details: None,
                },
            ),

            ApiError::Auth(AuthError::InvalidToken(msg)) => (
                StatusCode::UNAUTHORIZED,
                ApiErrorResponse {
                    code: "INVALID_TOKEN".to_string(),
                    message: format!("Invalid token: {}", msg),
                    details: None,
                },
            ),

            ApiError::Auth(AuthError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                ApiErrorResponse {
                    code: "INVALID_CREDENTIALS".to_string(),
                    message: "Invalid email or password".to_string(),
                    details: None,
                },
            ),

            ApiError::Auth(AuthError::BlacklistedToken) => (
                StatusCode::UNAUTHORIZED,
                ApiErrorResponse {
                    code: "TOKEN_BLACKLISTED".to_string(),
                    message: "Token has been invalidated".to_string(),
                    details: None,
                },
            ),

            ApiError::Auth(AuthError::InternalError(msg)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorResponse {
                    code: "AUTH_INTERNAL_ERROR".to_string(),
                    message: "Authentication process failed".to_string(),
                    details: Some(json!({ "reason": msg })),
                },
            ),

            ApiError::Auth(AuthError::DuplicateEmail) => (
                StatusCode::CONFLICT,
                ApiErrorResponse {
                    code: "EMAIL_ALREADY_EXISTS".to_string(),
                    message: "Email already exists".to_string(),
                    details: None,
                },
            ),

            // ── Validation & input errors ──
            ApiError::Validation(errors) => (
                StatusCode::BAD_REQUEST,
                ApiErrorResponse {
                    code: "VALIDATION_ERROR".to_string(),
                    message: "Invalid input data".to_string(),
                    details: Some(json!(errors)),
                },
            ),

            // ── Database & connection issues ──
            ApiError::Database(e) => match e {
                diesel::result::Error::NotFound => (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse {
                        code: "RESOURCE_NOT_FOUND".to_string(),
                        message: "Requested resource not found".to_string(),
                        details: None,
                    },
                ),

                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => (
                    StatusCode::CONFLICT,
                    ApiErrorResponse {
                        code: "CONFLICT".to_string(),
                        message: "Resource already exists (unique constraint violation)"
                            .to_string(),
                        details: None,
                    },
                ),

                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse {
                        code: "DATABASE_ERROR".to_string(),
                        message: "A database error occurred".to_string(),
                        details: None, // don't leak raw error
                    },
                ),
            },

            ApiError::DatabaseConnection(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiErrorResponse {
                    code: "DATABASE_UNAVAILABLE".to_string(),
                    message: "Database connection failed".to_string(),
                    details: None,
                },
            ),

            // ── Other specific domains ──
            ApiError::Argon2(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorResponse {
                    code: "PASSWORD_HASH_ERROR".to_string(),
                    message: "Password processing failed".to_string(),
                    details: None,
                },
            ),

            ApiError::Token(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorResponse {
                    code: "TOKEN_GENERATION_FAILED".to_string(),
                    message: "Failed to generate authentication token".to_string(),
                    details: None,
                },
            ),

            ApiError::Payment(msg) => (
                StatusCode::BAD_GATEWAY,
                ApiErrorResponse {
                    code: "PAYMENT_PROVIDER_ERROR".to_string(),
                    message: "Payment processing failed".to_string(),
                    details: Some(json!({ "reason": msg })),
                },
            ),

            ApiError::Webhook(e) => match e {
                WebhookError::BadSignature | WebhookError::BadTimestamp(_) => (
                    StatusCode::BAD_REQUEST,
                    ApiErrorResponse {
                        code: "INVALID_WEBHOOK_SIGNATURE".to_string(),
                        message: "Webhook signature verification failed".to_string(),
                        details: None,
                    },
                ),

                WebhookError::BadKey => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse {
                        code: "WEBHOOK_CONFIG_ERROR".to_string(),
                        message: "Webhook configuration error".to_string(),
                        details: None,
                    },
                ),

                _ => (
                    StatusCode::BAD_REQUEST,
                    ApiErrorResponse {
                        code: "WEBHOOK_ERROR".to_string(),
                        message: "Invalid webhook request".to_string(),
                        details: None,
                    },
                ),
            },

            // ── Generic fallback ──
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorResponse {
                    code: "INTERNAL_ERROR".to_string(),
                    message: "An unexpected error occurred".to_string(),
                    details: Some(json!({ "context": msg })), // optional – can be removed
                },
            ),

            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ApiErrorResponse {
                    code: "BAD_REQUEST".to_string(),
                    message: msg,
                    details: None,
                },
            ),
        };

        (status, Json(body)).into_response()
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
    DuplicateEmail,
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
            AuthError::DuplicateEmail => write!(f, "Email already exist"),
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
