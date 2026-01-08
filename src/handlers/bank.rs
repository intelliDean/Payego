use crate::config::security_config::Claims;
use crate::error::ApiError;
use crate::services::bank_service::BankService;
use crate::AppState;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use regex::Regex;
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
pub struct BankRequest {
    #[validate(length(min = 1, message = "Bank code is required"))]
    pub bank_code: String,
    #[validate(regex(
        path = "ACCOUNT_NUMBER_RE",
        message = "Account number must be 10 digits"
    ))]
    pub account_number: String,
    #[validate(length(min = 1, message = "Account name is required"))]
    pub bank_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct BankResponse {
    pub transaction_id: String,
    pub account_name: String,
}

static ACCOUNT_NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{10}$").expect("Invalid account number regex"));

#[utoipa::path(
    post,
    path = "/api/add_bank",
    request_body = BankRequest,
    responses(
        (status = 201, description = "Bank account added", body = BankResponse),
        (status = 400, description = "Invalid bank details"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Bank account already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Payment"
)]
pub async fn add_bank_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<BankRequest>,
) -> Result<Json<BankResponse>, (StatusCode, String)> {
    // Validate input
    req.validate().map_err(|e: validator::ValidationErrors| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Parse user_id
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e: uuid::Error| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    let response = BankService::add_bank_account(state, user_id, req)
        .await
        .map_err(|e| {
             let (status, msg) = e.into();
             (status, msg)
        })?;

    Ok(Json(response))
}
