mod common;

use payego::error::ApiError;
use http::StatusCode;
use diesel::result::Error as DieselError;
use validator::ValidationErrors;

#[test]
fn test_api_error_to_status_code_mapping() {
    // Database NotFound -> 401 Unauthorized
    let err = ApiError::Database(DieselError::NotFound);
    let (status, _): (StatusCode, String) = err.into();
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Database other error -> 500 Internal Server Error
    let err = ApiError::Database(DieselError::QueryBuilderError("broken".into()));
    let (status, _): (StatusCode, String) = err.into();
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);

    // Validation error -> 400 Bad Request
    let err = ApiError::Validation(ValidationErrors::new());
    let (status, _): (StatusCode, String) = err.into();
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Auth error -> 401 Unauthorized
    let err = ApiError::Auth("Token expired".to_string());
    let (status, _): (StatusCode, String) = err.into();
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Database connection error -> 500 Internal Server Error
    let err = ApiError::DatabaseConnection("Pool timeout".to_string());
    let (status, msg): (StatusCode, String) = err.into();
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(msg.contains("Database connection error"));
}

#[test]
fn test_api_error_display() {
    let err = ApiError::Auth("Unauthorized access".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Authentication error"));
    assert!(display.contains("Unauthorized access"));
}
