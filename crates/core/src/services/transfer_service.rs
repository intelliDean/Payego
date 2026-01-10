use diesel::prelude::*;
use diesel::dsl::sql;
use diesel::sql_types::BigInt;
use payego_primitives::error::ApiError;
use payego_primitives::models::{AppState, NewTransaction, Wallet, TransferRequest, WalletTransferRequest};
use payego_primitives::schema::{transactions, wallets};
use http::StatusCode;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
use secrecy::ExposeSecret;

pub struct TransferService;

impl TransferService {
    pub async fn transfer_internal(
        state: Arc<AppState>,
        sender_id: Uuid,
        req: WalletTransferRequest,
    ) -> Result<StatusCode, ApiError> {
        info!("Internal transfer initiated");

        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // 1. Idempotency check
        let existing_transaction = transactions::table
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("metadata->>'idempotency_key' = ")
                    .bind::<diesel::sql_types::Text, _>(&req.idempotency_key),
            )
            .filter(transactions::user_id.eq(sender_id))
            .first::<payego_primitives::models::Transaction>(&mut conn)
            .optional()
            .map_err(|e: diesel::result::Error| {
                error!("Database error checking idempotency: {}", e);
                ApiError::from(e)
            })?;

        if let Some(tx) = existing_transaction {
            info!(
                "Idempotent request: transaction {} already exists for key {}",
                tx.reference, req.idempotency_key
            );
            return Ok(StatusCode::OK);
        }

        let amount_cents = (req.amount * 100.0).round() as i64;

        conn.transaction::<StatusCode, ApiError, _>(|conn| {
            let sender_wallet = wallets::table
                .filter(wallets::user_id.eq(sender_id))
                .filter(wallets::currency.eq(&req.currency))
                .first::<Wallet>(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Sender wallet lookup failed: {}", e);
                    ApiError::from(e)
                })?;

            if sender_wallet.balance < amount_cents {
                return Err(ApiError::Payment("Insufficient balance".to_string()));
            }

            let recipient_wallet = wallets::table
                .filter(wallets::user_id.eq(req.recipient_id))
                .filter(wallets::currency.eq(&req.currency))
                .first::<Wallet>(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Recipient wallet lookup failed: {}", e);
                    ApiError::from(e)
                })?;

            diesel::update(wallets::table)
                .filter(wallets::id.eq(sender_wallet.id))
                .set(wallets::balance.eq(sql::<BigInt>("balance - ").bind::<BigInt, _>(amount_cents)))
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Sender debit failed: {}", e);
                    ApiError::from(e)
                })?;

            diesel::update(wallets::table)
                .filter(wallets::id.eq(recipient_wallet.id))
                .set(wallets::balance.eq(sql::<BigInt>("balance + ").bind::<BigInt, _>(amount_cents)))
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Recipient credit failed: {}", e);
                    ApiError::from(e)
                })?;

            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id: sender_id,
                    recipient_id: Some(req.recipient_id),
                    amount: -amount_cents,
                    transaction_type: "internal_transfer_send".to_string(),
                    status: "completed".to_string(),
                    provider: None,
                    description: Some(req.description.unwrap_or_else(|| "Internal transfer".to_string())),
                    reference: req.reference,
                    currency: req.currency.clone(),
                    metadata: Some(json!({
                        "idempotency_key": req.idempotency_key
                    })),
                })
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Sender transaction insert failed: {}", e);
                    ApiError::from(e)
                })?;

            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id: req.recipient_id,
                    recipient_id: Some(sender_id),
                    amount: amount_cents,
                    transaction_type: "internal_transfer_receive".to_string(),
                    status: "completed".to_string(),
                    provider: None,
                    description: Some("Received internal transfer".to_string()),
                    reference: Uuid::new_v4(), // recipient side doesn't need same reference or key
                    currency: req.currency,
                    metadata: None,
                })
                .execute(conn)
                .map_err(|e: diesel::result::Error| {
                    error!("Recipient transaction insert failed: {}", e);
                    ApiError::from(e)
                })?;

            Ok(StatusCode::OK)
        })
    }

    pub async fn transfer_external(
        state: Arc<AppState>,
        user_id: Uuid,
        req: TransferRequest,
    ) -> Result<StatusCode, ApiError> {
        info!("External transfer initiated");

        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database connection error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let sender_wallet = wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(&req.currency))
            .first::<Wallet>(&mut conn)
            .map_err(|e: diesel::result::Error| {
                error!("Sender wallet lookup failed: {}", e);
                ApiError::from(e)
            })?;

        let amount_ngn_cents = (req.amount * 100.0) as i64;
        let exchange_rate = if req.currency == "NGN" {
            1.0
        } else {
            Self::get_exchange_rate(&state.exchange_api_url, &req.currency, "NGN").await?
        };

        let amount_to_deduct = (amount_ngn_cents as f64 / exchange_rate).round() as i64;

        if sender_wallet.balance < amount_to_deduct {
            return Err(ApiError::Payment("Insufficient balance".to_string()));
        }

        let paystack_key = state.paystack_secret_key.expose_secret();
        let client = Client::new();
        let account_name = req.account_name.clone().unwrap_or_else(|| "External Recipient".to_string());

        let resp = client
            .post(format!("{}/transferrecipient", state.paystack_api_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&json!({
                "type": "nuban",
                "name": account_name,
                "account_number": req.account_number,
                "bank_code": req.bank_code,
                "currency": "NGN"
            }))
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Paystack error: {}", e)))?;

        let status = resp.status();
        let body = resp.json::<Value>().await.map_err(|e: reqwest::Error| ApiError::Payment(format!("Parsing error: {}", e)))?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
            return Err(ApiError::Payment("Paystack recipient creation failed".to_string()));
        }

        let recipient_code = body["data"]["recipient_code"].as_str().ok_or_else(|| ApiError::Payment("Missing code".to_string()))?.to_string();

        let resp = client
            .post(format!("{}/transfer", state.paystack_api_url))
            .header("Authorization", format!("Bearer {}", paystack_key))
            .json(&json!({
                "source": "balance",
                "amount": amount_ngn_cents,
                "recipient": recipient_code,
                "reference": req.reference.to_string(),
            }))
            .send()
            .await
            .map_err(|e: reqwest::Error| ApiError::Payment(format!("Transfer error: {}", e)))?;

        let status = resp.status();
        let body = resp.json::<Value>().await.map_err(|e: reqwest::Error| ApiError::Payment(format!("Parsing error: {}", e)))?;

        if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
            return Err(ApiError::Payment("Paystack transfer failed".to_string()));
        }

        let transfer_code = body["data"]["transfer_code"].as_str().unwrap_or("").to_string();

        conn.transaction::<StatusCode, ApiError, _>(|conn| {
            diesel::update(wallets::table)
                .filter(wallets::id.eq(sender_wallet.id))
                .set(wallets::balance.eq(sql::<BigInt>("balance - ").bind::<BigInt, _>(amount_to_deduct)))
                .execute(conn)
                .map_err(|e: diesel::result::Error| ApiError::from(e))?;

            diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    recipient_id: None,
                    amount: -amount_to_deduct,
                    transaction_type: "paystack_payout".to_string(),
                    currency: req.currency.clone(),
                    status: "pending".to_string(),
                    provider: Some("paystack".to_string()),
                    description: Some("External transfer".to_string()),
                    reference: req.reference,
                    metadata: Some(json!({
                        "transfer_code": transfer_code,
                        "idempotency_key": req.idempotency_key
                    })),
                })
                .execute(conn)
                .map_err(|e: diesel::result::Error| ApiError::from(e))?;

            Ok(StatusCode::OK)
        })
    }

    async fn get_exchange_rate(base_url: &str, from_currency: &str, to_currency: &str) -> Result<f64, ApiError> {
        let client = Client::new();
        let resp = client.get(format!("{}/{}", base_url, from_currency)).send().await.map_err(|e: reqwest::Error| ApiError::Payment(e.to_string()))?;
        let body = resp.json::<Value>().await.map_err(|e: reqwest::Error| ApiError::Payment(e.to_string()))?;
        body["rates"][to_currency].as_f64().ok_or_else(|| ApiError::Payment("Invalid rate".to_string()))
    }
}
