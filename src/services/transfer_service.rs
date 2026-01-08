use diesel::prelude::*;
use uuid::Uuid;
use crate::error::ApiError;
use crate::schema::{users, wallets, transactions};
use crate::models::models::{NewTransaction, Transaction};
use tracing::{error, info};

pub struct TransferService;

impl TransferService {
    pub fn execute_internal_transfer(
        conn: &mut PgConnection,
        sender_id: Uuid,
        recipient_email: &str,
        amount: f64,
        currency: &str,
        reference: Uuid,
    ) -> Result<String, ApiError> {
        // 1. Convert amount to cents
        let amount_cents = (amount * 100.0).round() as i64;
        let currency_upper = currency.to_uppercase();

        // 2. Lookup recipient
        let recipient_id = users::table
            .filter(users::email.eq(recipient_email))
            .select(users::id)
            .first::<Uuid>(conn)
            .map_err(|e| {
                error!("Recipient lookup failed: {}", e);
                if e == diesel::result::Error::NotFound {
                    ApiError::Payment("Recipient is not known".to_string())
                } else {
                    ApiError::Database(e)
                }
            })?;

        // 3. Prevent self-transfer
        if sender_id == recipient_id {
            error!("Self-transfer attempted: sender_id = {}", sender_id);
            return Err(ApiError::Auth("Cannot transfer to self".to_string()));
        }

        // 4. Idempotency check
        let existing_transaction = transactions::table
            .filter(transactions::reference.eq(reference))
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| {
                error!("Database error checking idempotency: {}", e);
                ApiError::Database(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!("Idempotent request: transaction {} already exists", tx.reference);
            return Ok(tx.reference.to_string());
        }

        // 5. Balance check
        let sender_balance = wallets::table
            .filter(wallets::user_id.eq(sender_id))
            .filter(wallets::currency.eq(&currency_upper))
            .select(wallets::balance)
            .first::<i64>(conn)
            .map_err(|e| {
                error!("Sender wallet lookup failed: {}", e);
                if e == diesel::result::Error::NotFound {
                    ApiError::Payment("Sender wallet not found for specified currency".to_string())
                } else {
                    ApiError::Database(e)
                }
            })?;

        if sender_balance < amount_cents {
            error!("Insufficient balance: available={}, required={}", sender_balance, amount_cents);
            return Err(ApiError::Payment("Insufficient balance".to_string()));
        }

        // 6. Atomic transaction
        conn.transaction(|conn| {
            // Debit sender
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(sender_id))
                .filter(wallets::currency.eq(&currency_upper))
                .set(wallets::balance.eq(wallets::balance - amount_cents))
                .execute(conn)
                .map_err(ApiError::Database)?;

            // Insert sender transaction
            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id: sender_id,
                    recipient_id: Some(recipient_id),
                    amount: -amount_cents,
                    transaction_type: "internal_transfer_send".to_string(),
                    status: "completed".to_string(),
                    provider: Some("internal".to_string()),
                    description: Some(format!("Transfer to {} in {}", recipient_email, currency_upper)),
                    reference,
                    currency: currency_upper.clone(),
                })
                .execute(conn)
                .map_err(ApiError::Database)?;

            // Credit recipient
            diesel::insert_into(wallets::table)
                .values((
                    wallets::user_id.eq(recipient_id),
                    wallets::balance.eq(amount_cents),
                    wallets::currency.eq(&currency_upper),
                ))
                .on_conflict((wallets::user_id, wallets::currency))
                .do_update()
                .set(wallets::balance.eq(wallets::balance + amount_cents))
                .execute(conn)
                .map_err(ApiError::Database)?;

            // Insert recipient transaction
            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id: recipient_id,
                    recipient_id: Some(sender_id),
                    amount: amount_cents,
                    transaction_type: "internal_transfer_receive".to_string(),
                    status: "completed".to_string(),
                    provider: Some("internal".to_string()),
                    description: Some(format!("Received from sender in {}", currency_upper)),
                    reference: Uuid::new_v4(), // Different reference for recipient tx?
                    currency: currency_upper,
                })
                .execute(conn)
                .map_err(ApiError::Database)?;

            Ok::<(), ApiError>(())
        })?;

        info!("Internal transfer completed: {} to {} from {} to {}", amount, recipient_email, sender_id, recipient_id);
        Ok(reference.to_string())
    }
}
