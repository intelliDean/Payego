use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Extension, Json};
use diesel::prelude::*;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::config::security_config::Claims;
use crate::models::user_models::{AppState, NewTransaction, Wallet};

#[utoipa::path(
    post,
    path = "/api/transfer",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer completed"),
        (status = 400, description = "Invalid recipient or insufficient balance")
    ),
    security(("Bearer" = []))
)]
pub async fn transfer(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TransferRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.db.get().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?;
    let sender_id = Uuid::parse_str(&claims.sub).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    let amount_cents = (req.amount * 100.0) as i64;

    // Validate recipient
    let recipient = crate::schema::users::table
        .filter(crate::schema::users::email.eq(&req.recipient_email))
        .select((crate::schema::users::id, crate::schema::users::email))
        .first::<(Uuid, String)>(&mut conn)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Recipient not found".to_string()))?;

    // Validate balance
    let sender_wallet = crate::schema::wallets::table
        .find(sender_id)
        .first::<Wallet>(&mut conn)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Wallet not found".to_string()))?;
    if sender_wallet.balance < amount_cents {
        return Err((StatusCode::BAD_REQUEST, "Insufficient balance".to_string()));
    }

    // Atomic transaction
    conn.transaction(|conn| {
        // Debit sender
        diesel::update(crate::schema::wallets::table.find(sender_id))
            .set((
                crate::schema::wallets::balance.eq(crate::schema::wallets::balance - amount_cents),
                crate::schema::wallets::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(conn)?;

        diesel::insert_into(crate::schema::transactions::table)
            .values(NewTransaction {
                user_id: sender_id,
                recipient_id: Some(recipient.0),
                amount: -amount_cents, // Negative for debit
                transaction_type: "internal_transfer_send".to_string(),
                status: "completed".to_string(),
                provider: None,
                description: Some(format!("Transfer to {}", req.recipient_email)),
                reference: Uuid::new_v4(),
            })
            .execute(conn)?;

        // Credit recipient
        diesel::update(crate::schema::wallets::table.find(recipient.0))
            .set((
                crate::schema::wallets::balance.eq(crate::schema::wallets::balance + amount_cents),
                crate::schema::wallets::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(conn)?;

        diesel::insert_into(crate::schema::transactions::table)
            .values(NewTransaction {
                user_id: recipient.0,
                recipient_id: Some(sender_id),
                amount: amount_cents,
                transaction_type: "internal_transfer_receive".to_string(),
                status: "completed".to_string(),
                provider: None,
                description: Some(format!("Transfer from {}", claims.sub)),
                reference: None,
            })
            .execute(conn)?;

        Ok(())
    })
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Transfer failed".to_string()))?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
pub struct TransferRequest {
    amount: f64, // In dollars
    recipient_email: String,
}