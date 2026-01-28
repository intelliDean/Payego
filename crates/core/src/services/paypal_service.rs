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
pub use crate::app_state::AppState;

use reqwest::Url;
use secrecy::ExposeSecret;
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

        let token = resp
            .json::<PayPalTokenResponse>()
            .await
            .map_err(|_| ApiError::Payment("Invalid PayPal token response".into()))?;

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
        let mut conn = state.db.get().map_err(|e| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        let transaction =
            TransactionRepository::find_by_id_or_reference(&mut conn, transaction_ref)?
                .ok_or_else(|| ApiError::Payment("Transaction not found".into()))?;

        // ── Idempotency
        if transaction.txn_state == PaymentState::Completed {
            return Ok(CaptureResponse {
                status: PaymentState::Completed,
                transaction_id: transaction_ref,
                error_message: None,
            });
        }

        if transaction.txn_state != PaymentState::Pending {
            return Err(ApiError::Payment("Invalid transaction state".into()));
        }

        let capture = Self::paypal_capture_api(state, &order_id).await?;

        if capture.currency != transaction.currency {
            return Err(ApiError::Payment("Currency mismatch".into()));
        }

        conn.transaction::<_, ApiError, _>(|conn| {
            // ── Update transaction
            TransactionRepository::update_status_and_provider_ref(
                conn,
                transaction.id,
                PaymentState::Completed,
                Some(capture.capture_id.clone()),
            )?;

            // ── Lock wallet
            let wallet = WalletRepository::find_by_user_and_currency_with_lock(
                conn,
                transaction.user_id,
                transaction.currency,
            )?;

            // ── Ledger entry
            WalletRepository::add_ledger_entry(
                conn,
                NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: transaction.id,
                    amount: transaction.amount,
                },
            )?;

            // ── Update balance
            WalletRepository::credit(conn, wallet.id, transaction.amount)?;

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
        let client = state.http_client.clone();
        let token = Self::get_access_token(state).await?;

        let base = Url::parse(&state.config.paypal_details.paypal_api_url)
            .map_err(|_| ApiError::Internal("Invalid PayPal base URL".into()))?;

        let url = base
            .join(&format!("v2/checkout/orders/{}/capture", order_id))
            .map_err(|_| ApiError::Internal("Invalid PayPal capture URL".into()))?;

        let resp = client
            .post(url)
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "PayPal capture request failed");
                ApiError::Payment("Failed to reach PayPal".into())
            })?
            .error_for_status()
            .map_err(|e| {
                tracing::warn!(error = %e, "PayPal capture rejected");
                ApiError::Payment("PayPal capture rejected".into())
            })?;

        let body: PayPalCaptureResponse = resp.json().await.map_err(|e| {
            tracing::error!(error = %e, "Invalid PayPal capture response");
            ApiError::Payment("Invalid PayPal capture response".into())
        })?;

        let capture = body
            .purchase_units
            .first()
            .and_then(|pu| pu.payments.captures.first())
            .ok_or_else(|| ApiError::Payment("Missing PayPal capture data".into()))?;

        let currency = CurrencyCode::from_str(&capture.amount.currency_code)
            .map_err(|_| ApiError::Payment("Unsupported currency".into()))?;

        Ok(PaypalCapture {
            capture_id: capture.id.clone(),
            currency,
        })
    }
}
