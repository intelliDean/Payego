use crate::config::security_config::Claims;
use crate::models::user_models::{NewTransaction, Wallet};
use crate::schema::wallets::user_id;
use crate::schema::{transactions, users, wallets};
use crate::{AppState, error::ApiError};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/transfer/internal",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer completed"),
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
) -> Result<StatusCode, (StatusCode, String)> {
    info!(
        "Transfer request: sender = {}, recipient_email = {}, amount = {}",
        claims.sub, req.recipient_email, req.amount
    );

    // Validate amount
    if req.amount <= 0.0 {
        error!("Invalid amount: {}", req.amount);
        return Err(ApiError::Auth("Amount must be positive".to_string()).into());
    }
    let amount_cents = (req.amount * 100.0).round() as i64; // Round to avoid floating-point issues
    debug!(
        "Converted amount: {} dollars to {} cents",
        req.amount, amount_cents
    );

    let conn = &mut state
        .db
        .get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

    // Parse sender ID
    let sender_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID: {}", e);
        ApiError::Auth("Invalid user ID".to_string())
    })?;

    // Validate recipient
    let recipient = users::table
        .filter(users::email.eq(&req.recipient_email))
        .select(Recipient::as_select())
        .first(conn)
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

    // Validate sender wallet
    let sender_wallet = wallets::table
        .filter(wallets::user_id.eq(sender_id))
        .filter(wallets::currency.eq(&req.currency))
        .first::<Wallet>(conn)
        .map_err(|e| {
            error!("Sender wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Sender wallet not found".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    // Validate balance
    if sender_wallet.balance < amount_cents {
        error!(
            "Insufficient balance: available={}, required={}",
            sender_wallet.balance, amount_cents
        );
        return Err(ApiError::Auth("Insufficient balance".to_string()).into());
    }

    info!("Recipient ID: {}", recipient.id);
    // Validate recipient wallet and currency match
    let recipient_wallet = wallets::table
        .filter(wallets::user_id.eq(recipient.id))
        .filter(wallets::currency.eq(&req.currency))
        .first::<Wallet>(conn)
        .map_err(|e| {
            error!("Recipient wallet lookup failed: {}", e);
            if e.to_string().contains("not found") {
                ApiError::Payment("Recipient wallet not found".to_string())
            } else {
                ApiError::Database(e)
            }
        })?;

    if sender_wallet.currency != recipient_wallet.currency {
        error!(
            "Currency mismatch: sender_currency = {}, recipient_currency = {}",
            sender_wallet.currency, recipient_wallet.currency
        );
        return Err(
            ApiError::Auth("Sender and recipient must use the same currency".to_string()).into(),
        );
    }

    // Atomic transaction
    conn.transaction(|conn| {
        // Debit sender
        diesel::update(wallets::table.filter(user_id.eq(sender_id)))
            .set((
                wallets::balance.eq(wallets::balance - amount_cents),
                // wallets::updated_at.eq(Utc::now()),
            ))
            .execute(conn)
            .map_err(|e| {
                error!("Sender wallet update failed: {}", e);
                ApiError::Database(e)
            })?;
        info!(
            "Debited sender wallet: user_id={}, amount={}",
            sender_id, amount_cents
        );

        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id: sender_id,
                recipient_id: Some(recipient.id),
                amount: -amount_cents,
                transaction_type: "internal_transfer_send".to_string(),
                status: "completed".to_string(),
                provider: Option::from("internal".to_string()),
                description: Some(format!("Transfer to user {}", recipient.id)),
                reference: Uuid::new_v4(),
            })
            .execute(conn)
            .map_err(|e| {
                error!("Sender transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        // Credit recipient
        diesel::update(wallets::table.filter(user_id.eq(recipient.id)))
            .set(wallets::balance.eq(wallets::balance + amount_cents))
            .execute(conn)
            .map_err(|e| {
                error!("Recipient wallet update failed: {}", e);
                ApiError::Database(e)
            })?;
        info!(
            "Credited recipient wallet: user_id={}, amount={}",
            recipient.id, amount_cents
        );

        diesel::insert_into(transactions::table)
            .values(NewTransaction {
                user_id: recipient.id,
                recipient_id: Some(sender_id),
                amount: amount_cents,
                transaction_type: "internal_transfer_receive".to_string(),
                status: "completed".to_string(),
                provider: Option::from("internal".to_string()),
                description: Some(format!("Transfer from user {}", sender_id)),
                reference: Uuid::new_v4(),
            })
            .execute(conn)
            .map_err(|e| {
                error!("Recipient transaction insert failed: {}", e);
                ApiError::Database(e)
            })?;

        Ok::<(), ApiError>(())
    })?;

    info!(
        "Transfer completed: sender={}, recipient={}, amount={}",
        sender_id, recipient.id, amount_cents
    );
    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
pub struct TransferRequest {
    amount: f64, // In base units of currency
    recipient_email: String,
    currency: String, // e.g., "USD", "NGN"
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct Recipient {
    pub id: Uuid,
    pub email: String,
}
