

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

    // Convert amount to cents
    let amount_cents = (req.amount * 100.0).round() as i64;

    // Get database connection
    let conn = &mut state
        .db
        .get()
        .map_err(|e| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

    // Parse recipient
    let recipient = users::table
        .filter(users::email.eq(&req.recipient_email))
        .select(Recipient::as_select())
        .first::<Recipient>(conn)
        .map_err(|e| {
            error!("Recipient lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Recipient is not known".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    // Prevent self-transfer
    if sender_id == recipient.id {
        error!("Self-transfer attempted: sender_id = {}", sender_id);
        return Err(ApiError::Auth("Cannot transfer to self".to_string()).into());
    }

    // Idempotency check: check if transaction with this reference already exists
    let existing_transaction = transactions::table
        .filter(transactions::reference.eq(req.reference))
        .first::<crate::models::models::Transaction>(conn)
        .optional()
        .map_err(|e| {
            error!("Database error checking idempotency: {}", e);
            ApiError::Database(e)
        })?;

    if let Some(tx) = existing_transaction {
        info!("Idempotent request: transaction {} already exists", tx.reference);
        return Ok(Json(TransferResponse {
            transaction_id: tx.reference.to_string(),
        }));
    }

    // Validate sender wallet and balance
    let sender_wallet = wallets::table
        .filter(wallets::user_id.eq(sender_id))
        .filter(wallets::currency.eq(&req.currency.to_uppercase()))
        .select((wallets::balance, wallets::currency))
        .first::<(i64, String)>(conn)
        .map_err(|e| {
            error!("Sender wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Sender wallet not found for specified currency".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    if sender_wallet.0 < amount_cents {
        error!(
            "Insufficient balance: available={}, required={}",
            sender_wallet.0, amount_cents
        );
        return Err((
            StatusCode::BAD_REQUEST,
            "Insufficient balance".to_string(),
        ));
    }

    // Perform transfer atomically
    let transaction_reference = req.reference;
    conn.transaction(|conn| {
        // Debit sender's wallet
        diesel::update(wallets::table)
            .filter(wallets::user_id.eq(sender_id))
            .filter(wallets::currency.eq(&req.currency.to_uppercase()))
            .set(wallets::balance.eq(wallets::balance - amount_cents))
            .execute(conn)
            .map_err(|e| {
                error!("Sender wallet update failed: {}", e);
                ApiError::Database(e)
            })?;

        // Create sender transaction
        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id: sender_id,
                recipient_id: Some(recipient.id),
                amount: -amount_cents, // Negative for sender
                transaction_type: "internal_transfer_send".to_string(),
                status: "completed".to_string(),
                provider: Some("internal".to_string()),
                description: Some(format!("Transfer to {} in {}", req.recipient_email, req.currency)),
                reference: transaction_reference,
                currency: req.currency.to_uppercase(),
            })
            .execute(conn)
            .map_err(|e| {
                error!("Sender transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        // Credit recipient's wallet (create or update)
        diesel::insert_into(wallets::table)
            .values((
                wallets::user_id.eq(recipient.id),
                wallets::balance.eq(amount_cents),
                wallets::currency.eq(req.currency.to_uppercase()),
            ))
            .on_conflict((wallets::user_id, wallets::currency))
            .do_update()
            .set(wallets::balance.eq(wallets::balance + amount_cents))
            .execute(conn)
            .map_err(|e| {
                error!("Recipient wallet update failed: {}", e);
                ApiError::Database(e)
            })?;

        // Create recipient transaction
        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id: recipient.id,
                recipient_id: Some(sender_id),
                amount: amount_cents,
                transaction_type: "internal_transfer_receive".to_string(),
                status: "completed".to_string(),
                provider: Some("internal".to_string()),
                description: Some(format!("Received from sender in {}", req.currency)),
                reference: Uuid::new_v4(),
                currency: req.currency.to_uppercase(),
            })
            .execute(conn)
            .map_err(|e| {
                error!("Recipient transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        Ok::<(), ApiError>(())
    })?;

    info!(
        "Internal transfer completed: {} {} from {} to {}",
        req.amount, req.currency, sender_id, recipient.id
    );

    Ok(Json(TransferResponse {
        transaction_id: transaction_reference.to_string(),
    }))
}