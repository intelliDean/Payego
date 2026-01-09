use crate::error::ApiError;
use crate::models::models::PayoutRequest;
use crate::models::models::{AppState, NewTransaction, Transaction, Wallet};
use crate::schema::{transactions, users, wallets};
use diesel::prelude::*;
use reqwest::{Client, StatusCode};
use secrecy::ExposeSecret;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct TransferService;

impl TransferService {
    pub fn execute_internal_transfer(
        conn: &mut PgConnection,
        sender_id: Uuid,
        recipient_email: &str,
        amount: f64,
        currency: &str,
        reference: Uuid,
        idempotency_key: &str,
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

        // 4. Idempotency check with metadata
        let existing_transaction = transactions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ")
                    .bind::<diesel::sql_types::Text, _>(idempotency_key),
            )
            .filter(transactions::user_id.eq(sender_id))
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| {
                error!("Database error checking idempotency: {}", e);
                ApiError::Database(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!(
                "Idempotent request: transaction {} already exists for key {}",
                tx.reference, idempotency_key
            );
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
            error!(
                "Insufficient balance: available={}, required={}",
                sender_balance, amount_cents
            );
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
                    description: Some(format!(
                        "Transfer to {} in {}",
                        recipient_email, currency_upper
                    )),
                    reference,
                    currency: currency_upper.clone(),
                    metadata: Some(json!({
                        "idempotency_key": idempotency_key
                    })),
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
                    metadata: None, // Recipient doesn't track sender's idempotency key usually
                })
                .execute(conn)
                .map_err(ApiError::Database)?;

            Ok::<(), ApiError>(())
        })?;

        info!(
            "Internal transfer completed: {} to {} from {} to {}",
            amount, recipient_email, sender_id, recipient_id
        );
        Ok(reference.to_string())
    }

    pub async fn execute_external_transfer(
        state: Arc<AppState>,
        user_id: Uuid,
        req: PayoutRequest,
    ) -> Result<StatusCode, ApiError> {
        let amount_ngn_cents = (req.amount * 100.0).round() as i64; // Amount in NGN cents

        let mut conn = state.db.get().map_err(|e| {
            error!("Database connection failed: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // ALL LOGIC FROM HANDLER GOES HERE
        // Fetch sender wallet in the selected currency
        let sender_wallet = wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(&req.currency))
            .first::<Wallet>(&mut conn)
            .map_err(|e| {
                error!("Sender wallet lookup failed: {}", e);
                if e == diesel::result::Error::NotFound {
                    ApiError::Payment(format!(
                        "Sender wallet not found for currency {}",
                        req.currency
                    ))
                } else {
                    ApiError::Database(e).into()
                }
            })?;

        // Fetch current exchange rate (use a tool or API)
        let exchange_rate = Self::get_exchange_rate(&state.exchange_api_url, &req.currency, "NGN")
            .await
            .map_err(|e| {
                // error!("Exchange rate fetch failed: {}", e);
                ApiError::Payment("Exchange rate fetch failed".to_string())
            })?;

        let amount_to_deduct = (amount_ngn_cents as f64 / exchange_rate).round() as i64;

        // Validate balance
        if sender_wallet.balance < amount_to_deduct {
            error!(
                "Insufficient balance: available={}, required={}",
                sender_wallet.balance, amount_to_deduct
            );
            return Err(ApiError::Auth("Insufficient balance".to_string()).into());
        }

        // Create temporary Paystack recipient
        let paystack_key = state.paystack_secret_key.expose_secret();
        let client = Client::new();
        let account_name = req
            .account_name
            .clone()
            .unwrap_or("External Transfer Recipient".to_string());

        // PAYSTACK CALLS (RECIPIENT)
        let resp = client
            .post("https://api.paystack.co/transferrecipient")
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&serde_json::json!({
                "type": "nuban",
                "name": account_name,
                "account_number": req.account_number,
                "bank_code": req.bank_code,
                "currency": "NGN"
            }))
            .send()
            .await
            .map_err(|e| {
                error!("Paystack recipient creation failed: {}", e);
                ApiError::Payment(format!("Paystack recipient creation failed: {}", e))
            })?;

        let status = resp.status();
        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            error!("Paystack response parsing error: {}", e);
            ApiError::Payment(format!("Paystack response error: {}", e))
        })?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
            let message = body["message"]
                .as_str()
                .unwrap_or("Unknown Paystack error")
                .to_string();
            error!("Paystack recipient creation failed: {}", message);
            return Err(ApiError::Payment(format!(
                "Paystack recipient creation failed: {}",
                message
            ))
            .into());
        }

        // Idempotency check with metadata
        let existing_transaction = transactions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ")
                    .bind::<diesel::sql_types::Text, _>(&req.idempotency_key),
            )
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error checking idempotency: {}", e);
                ApiError::Database(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!(
                "Idempotent request: transaction {} already exists for key {}",
                tx.reference, req.idempotency_key
            );
            return Ok(StatusCode::OK);
        }

        let recipient_code = body["data"]["recipient_code"]
            .as_str()
            .ok_or_else(|| {
                error!("Invalid Paystack response: missing recipient_code");
                ApiError::Payment("Invalid Paystack response: missing recipient_code".to_string())
            })?
            .to_string();
        debug!("Paystack recipient_code: {}", recipient_code);

        // Initiate Paystack transfer
        let reference = req.reference;
        let resp = client
            .post("https://api.paystack.co/transfer")
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&serde_json::json!({
                "source": "balance",
                "reason": format!("External transfer from Payego in {}", req.currency),
                "amount": amount_ngn_cents,
                "recipient": recipient_code,
                "reference": reference.to_string(),
            }))
            .send()
            .await
            .map_err(|e| {
                error!("Paystack transfer failed: {}", e);
                ApiError::Payment(format!("Paystack transfer failed: {}", e))
            })?;

        let status = resp.status();
        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            error!("Paystack response parsing error: {}", e);
            ApiError::Payment(format!("Paystack response error: {}", e))
        })?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
            let message = body["message"]
                .as_str()
                .unwrap_or("Unknown Paystack error")
                .to_string();
            error!("Paystack transfer failed: {}", message);
            return Err(ApiError::Payment(format!("Paystack transfer failed: {}", message)).into());
        }

        let transfer_code = body["data"]["transfer_code"]
            .as_str()
            .ok_or_else(|| {
                error!("Invalid Paystack response: missing transfer_code");
                ApiError::Payment("Invalid Paystack response: missing transfer_code".to_string())
            })?
            .to_string();
        debug!("Paystack transfer_code: {}", transfer_code);

        // Atomic transaction
        conn.transaction(|conn| {
            // Debit sender wallet
            diesel::update(wallets::table)
                .filter(wallets::user_id.eq(user_id))
                .filter(wallets::currency.eq(&req.currency))
                .set(wallets::balance.eq(wallets::balance - amount_to_deduct))
                .execute(conn)
                .map_err(|e| {
                    error!("Sender wallet update failed: {}", e);
                    ApiError::Database(e)
                })?;
            info!(
                "Debited sender wallet: user_id={}, amount={}",
                user_id, amount_to_deduct
            );

            // Insert transaction
            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: -amount_to_deduct,
                    transaction_type: "paystack_payout".to_string(),
                    currency: req.currency.to_uppercase(),
                    status: "pending".to_string(),
                    provider: Some("paystack".to_string()),
                    description: Some(format!("External transfer in {} to bank", &req.currency)),
                    reference,
                    metadata: Some(json!({
                        "transfer_code": transfer_code,
                        "exchange_rate": exchange_rate,
                        "idempotency_key": req.idempotency_key
                    })),
                })
                .execute(conn)
                .map_err(|e| {
                    error!("Payout transaction insert failed: {}", e);
                    ApiError::Database(e)
                })?;

            Ok::<(), ApiError>(())
        })?;

        info!(
            "External transfer initiated: user_id={}, amount_ngn={}, currency={}, exchange_rate={}",
            user_id, req.amount, req.currency, exchange_rate
        );
        Ok(StatusCode::OK)
    }

    async fn get_exchange_rate(
        base_url: &str,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<f64, ApiError> {
        let url = format!("{}/{}", base_url, from_currency);
        let client = Client::new();
        let resp = client.get(url).send().await.map_err(|e| {
            error!("Exchange rate API error: {}", e);
            ApiError::Payment(format!("Exchange rate API error: {}", e))
        })?;

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            error!("Exchange rate response parsing error: {}", e);
            ApiError::Payment(format!("Exchange rate response error: {}", e))
        })?;

        let rate = body["rates"][to_currency].as_f64().ok_or_else(|| {
            error!("Invalid exchange rate response");
            ApiError::Payment("Invalid exchange rate response".to_string())
        })?;

        Ok(rate)
    }
}
