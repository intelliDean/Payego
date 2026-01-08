

use axum::{extract::{State, Extension}, Json, http::StatusCode};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;
use validator::{Validate, ValidationError};
use regex::Regex;
use lazy_static::lazy_static;
use crate::AppState;
use crate::error::ApiError;
use crate::config::security_config::Claims;
use crate::schema::{users, wallets, transactions};
use crate::models::models::NewTransaction;
use tracing::{error, info};
use utoipa::ToSchema;

static SUPPORTED_CURRENCIES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(USD|NGN|GBP|EUR|CAD|AUD|JPY|CHF|CNY|SEK|NZD|MXN|SGD|HKD|NOK|KRW|TRY|INR|BRL|ZAR)$",
    )
        .expect("Invalid currency")
});

#[derive(Deserialize, ToSchema, Validate)]
pub struct TransferRequest {
    #[validate(range(
        min = 1.0,
        max = 10000.0,
        message = "Amount must be between 1 and 10,000"
    ))]
    pub amount: f64, // In base units (e.g., dollars)
    #[validate(email)]
    pub recipient_email: String,
    #[validate(regex(path = "SUPPORTED_CURRENCIES", message = "Invalid currency"))]
    pub currency: String,
    pub reference: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct TransferResponse {
    pub transaction_id: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct Recipient {
    pub id: Uuid,
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/api/transfer/internal",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer completed successfully", body = TransferResponse),
        (status = 400, description = "Invalid recipient, insufficient balance, or invalid amount"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = [])),
    tag = "Transaction"
)]
pub async fn internal_transfer(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, String)> {
    info!(
        "Transfer request: sender = {}, recipient_email = {}, amount = {}",
        claims.sub, req.recipient_email, req.amount
    );

    // Validate input
    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    // Parse sender ID
    let sender_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Get database connection
    let mut conn = state
        .db
        .get()
        .map_err(|e| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

    // Execute transfer via service
    let transaction_id = crate::services::transfer_service::TransferService::execute_internal_transfer(
        &mut conn,
        sender_id,
        &req.recipient_email,
        req.amount,
        &req.currency,
        req.reference,
    )?;

    Ok(Json(TransferResponse {
        transaction_id,
    }))
}