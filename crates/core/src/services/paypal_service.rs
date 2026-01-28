pub use crate::app_state::AppState;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
use diesel::Connection;
use http::StatusCode;
use payego_primitives::models::dtos::providers::paypal::{
    CaptureResponse, PayPalCaptureResponse, PayPalOrderResponse, PayPalTokenResponse, PaypalCapture,
};
pub use payego_primitives::{
    error::ApiError,
    models::{
        enum_types::{CurrencyCode, PaymentProvider, PaymentState},
        transaction::Transaction,
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
};

use reqwest::Url;
use secrecy::ExposeSecret;
use serde_json;
use std::str::FromStr;
use std::time::Duration;
use tracing::log::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct PayPalService;

impl PayPalService {
    async fn get_access_token(state: &AppState) -> Result<String, ApiError> {
        let base = Url::parse(&state.config.paypal_details.paypal_api_url)
            .map_err(|_| ApiError::Internal("Invalid PayPal base URL".into()))?;

        let url = base
            .join("v1/oauth2/token")
            .map_err(|_| ApiError::Internal("Invalid PayPal token URL".into()))?;

        let client_id = &state.config.paypal_details.paypal_client_id;
        let secret = state.config.paypal_details.paypal_secret.expose_secret();

        //this is just to double-check as client_id  and secret will never be empty
        if client_id.trim().is_empty() || secret.trim().is_empty() {
            return Err(ApiError::Internal(
                "PayPal credentials not configured".into(),
            ));
        }

        let resp = state
            .http_client
            .post(url)
            .basic_auth(client_id, Some(secret))
            .form(&[("grant_type", "client_credentials")])
            .timeout(Duration::from_secs(30)) // Increased to 30s for slow Sandbox responses
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "PayPal auth request failed");
                ApiError::Payment("Unable to reach PayPal".into())
            })?;

        if resp.status() != StatusCode::OK {
            tracing::warn!(
                status = %resp.status(),
                "PayPal auth rejected"
            );
            return Err(ApiError::Payment("PayPal authentication failed".into()));
        }

        let token = resp.json::<PayPalTokenResponse>().await.map_err(|e| {
            tracing::error!("Failed to parse PayPal token response: {}", e);
            ApiError::Payment("Invalid PayPal token response".into())
        })?;

        tracing::info!("Successfully retrieved PayPal access token");

        Ok(token.access_token)
    }

    pub async fn get_order_status(state: &AppState, order_id: &str) -> Result<String, ApiError> {
        let token = Self::get_access_token(state).await?;

        let base = Url::parse(&state.config.paypal_details.paypal_api_url)
            .map_err(|_| ApiError::Internal("Invalid PayPal base URL".into()))?;

        let url = base
            .join("v2/checkout/orders/")
            .and_then(|u| u.join(order_id))
            .map_err(|_| ApiError::Internal("Invalid PayPal order URL".into()))?;

        let resp = state
            .http_client
            .get(url)
            .bearer_auth(token)
            .timeout(Duration::from_secs(30)) // Increased to 30s
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "PayPal order fetch failed");
                ApiError::Payment("Failed to reach PayPal".into())
            })?;

        if !resp.status().is_success() {
            tracing::warn!(
                status = %resp.status(),
                "PayPal order fetch rejected"
            );
            return Err(ApiError::Payment("Failed to fetch PayPal order".into()));
        }

        let order = resp.json::<PayPalOrderResponse>().await.map_err(|e| {
            tracing::error!(error = %e, "Invalid PayPal order response");
            ApiError::Payment("Invalid PayPal order response".into())
        })?;

        Ok(order.status)
    }

    pub async fn capture_order(
        state: &AppState,
        order_id: String,
        transaction_ref: Uuid,
    ) -> Result<CaptureResponse, ApiError> {
        tracing::info!(
            "Starting capture_order: order_id={}, transaction_ref={}",
            order_id,
            transaction_ref
        );

        let mut conn = state.db.get().map_err(|e| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let transaction =
            TransactionRepository::find_by_id_or_reference(&mut conn, transaction_ref)?
                .ok_or_else(|| {
                    tracing::error!("Transaction not found: {}", transaction_ref);
                    ApiError::Payment("Transaction not found".into())
                })?;

        tracing::info!(
            "Found transaction: id={}, state={:?}",
            transaction.id,
            transaction.txn_state
        );

        // ── Idempotency
        if transaction.txn_state == PaymentState::Completed {
            tracing::info!("Transaction already completed, returning idempotent response");
            return Ok(CaptureResponse {
                status: PaymentState::Completed,
                transaction_id: transaction_ref,
                error_message: None,
            });
        }

        if transaction.txn_state != PaymentState::Pending {
            tracing::error!(
                "Invalid transaction state for capture: {:?}",
                transaction.txn_state
            );
            return Err(ApiError::Payment("Invalid transaction state".into()));
        }

        let capture = Self::paypal_capture_api(state, &order_id).await?;

        if capture.currency != transaction.currency {
            tracing::error!(
                "Currency mismatch: expected {:?}, got {:?}",
                transaction.currency,
                capture.currency
            );
            return Err(ApiError::Payment("Currency mismatch".into()));
        }

        conn.transaction::<_, ApiError, _>(|conn| {
            tracing::info!("Starting database transaction for capture");

            // ── Update transaction
            TransactionRepository::update_status_and_provider_ref(
                conn,
                transaction.id,
                PaymentState::Completed,
                Some(capture.capture_id.clone()),
            )?;
            tracing::info!("Transaction updated to Completed");

            // ── Lock wallet or create if needed
            // Use create_if_not_exists to avoid "Wallet not found" error
            let wallet = WalletRepository::create_if_not_exists(
                conn,
                transaction.user_id,
                transaction.currency,
            )?;
            tracing::info!("Wallet retrieved/created: {}", wallet.id);

            // ── Ledger entry
            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: transaction.id,
                    amount: transaction.amount,
                },
            )?;
            tracing::info!("Ledger entry added");

            // ── Update balance
            WalletRepository::credit(conn, wallet.id, transaction.amount)?;
            tracing::info!("Wallet credited");

            Ok(())
        })?;

        Ok(CaptureResponse {
            status: PaymentState::Completed,
            transaction_id: transaction_ref,
            error_message: None,
        })
    }

    async fn paypal_capture_api(
        state: &AppState,
        order_id: &str,
    ) -> Result<PaypalCapture, ApiError> {
        tracing::info!("Retrieving PayPal access token...");
        let token = Self::get_access_token(state).await?;

        let base = Url::parse(&state.config.paypal_details.paypal_api_url)
            .map_err(|_| ApiError::Internal("Invalid PayPal base URL".into()))?;

        let url = base
            .join(&format!("v2/checkout/orders/{}/capture", order_id))
            .map_err(|_| ApiError::Internal("Invalid PayPal capture URL".into()))?;

        let request = state
            .http_client
            .post(url)
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .header("PayPal-Request-Id", Uuid::new_v4().to_string());

        let resp = request.send().await.map_err(|e| {
            tracing::error!(error = %e, "PayPal capture request failed");
            ApiError::Payment("Failed to reach PayPal".into())
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            tracing::error!(status = %status, body = %body, "PayPal capture rejected");
            return Err(ApiError::Payment(format!(
                "PayPal capture rejected: {}",
                body
            )));
        }

        // Read raw body first to debug schema mismatches
        let body_text = resp.text().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to read response text");
            ApiError::Payment("Failed to read PayPal response".into())
        })?;

        let body: PayPalCaptureResponse = serde_json::from_str(&body_text).map_err(|e| {
            tracing::error!(error = %e, "Invalid PayPal capture response schema");
            ApiError::Payment(format!("Invalid PayPal capture response: {}", e))
        })?;

        tracing::info!("Deserialization successful: {:?}", body);

        let capture = body
            .purchase_units
            .first()
            .and_then(|pu| pu.payments.captures.first())
            .ok_or_else(|| {
                tracing::error!("Missing capture data in PayPal response: {:?}", body);
                ApiError::Payment("Missing PayPal capture data".into())
            })?;

        tracing::info!("Capture extracted: {:?}", capture);

        let currency = CurrencyCode::from_str(&capture.amount.currency_code).map_err(|e| {
            tracing::error!(
                "Unsupported currency code '{}': {}",
                capture.amount.currency_code,
                e
            );
            ApiError::Payment("Unsupported currency".into())
        })?;

        tracing::info!("Currency parsed: {:?}", currency);

        Ok(PaypalCapture {
            capture_id: capture.id.clone(),
            currency,
        })
    }
}
