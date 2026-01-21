use crate::error::ApiError;
use eyre::Report;
use secrecy::SecretString;
use std::env;

#[derive(Clone, Debug)]
pub struct JWTInfo {
    pub jwt_secret: SecretString,
    pub jwt_expiration_hours: i64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
}

impl JWTInfo {
    pub fn new() -> Result<JWTInfo, Report> {
        let jwt_secret =
            env::var("JWT_SECRET").expect("JWT_SECRET must be set in environment variables");

        if jwt_secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 characters long");
        }

        Ok(Self {
            jwt_secret: SecretString::new(jwt_secret.into()),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "2".into())
                .parse()
                .map_err(|e| {
                    ApiError::Token(format!("Invalid JWT expiration configuration: {}", e))
                })?,

            jwt_issuer: env::var("ISSUER").map_err(|e| {
                ApiError::Token(format!("Issuer environment variable not set: {}", e))
            })?,

            jwt_audience: env::var("AUDIENCE").map_err(|e| {
                ApiError::Token(format!("Audience environment variable not set: {}", e))
            })?,
        })
    }
}
