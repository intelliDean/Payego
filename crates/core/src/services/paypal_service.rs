use diesel::{
    ExpressionMethods, {Connection, RunQueryDsl}, {OptionalExtension, QueryDsl},
};
use http::StatusCode;
pub use payego_primitives::{
    error::ApiError,
    models::{
        app_state::app_state::AppState,
        dtos::providers_dto::{CaptureResponse, PayPalOrderResponse, PayPalTokenResponse},
        enum_types::{CurrencyCode, PaymentProvider, PaymentState},
        providers_dto::CaptureRequest,
        providers_dto::OrderResponse,
        providers_dto::PaypalCapture,
        transaction::Transaction,
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
    schema::{transactions, wallet_ledger, wallets},
};
use reqwest::Url;
use secrecy::ExposeSecret;
use std::str::FromStr;
use std::time::Duration;
use tracing::log::error;
use uuid::Uuid;
use payego_primitives::models::providers_dto::PayPalCaptureResponse;

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
            .timeout(Duration::from_secs(5)) // to override the default which I set to be 10
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
            .timeout(Duration::from_secs(5)) //override the default
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

        let transaction = transactions::table
            .filter(transactions::reference.eq(transaction_ref))
            .first::<Transaction>(&mut conn)
            .optional()?
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
            diesel::update(transactions::table)
                .filter(transactions::id.eq(transaction.id))
                .set((
                    transactions::txn_state.eq(PaymentState::Completed),
                    transactions::provider.eq(Some(PaymentProvider::Paypal)),
                    transactions::provider_reference.eq(Some(capture.capture_id.clone())),
                ))
                .execute(conn)?;

            // ── Lock wallet
            let wallet = wallets::table
                .filter(wallets::user_id.eq(transaction.user_id))
                .filter(wallets::currency.eq(transaction.currency))
                .for_update()
                .first::<Wallet>(conn)?;

            // ── Ledger entry
            diesel::insert_into(wallet_ledger::table)
                .values(NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: transaction.id,
                    amount: transaction.amount,
                })
                .execute(conn)?;

            // ── Update balance
            diesel::update(wallets::table)
                .filter(wallets::id.eq(wallet.id))
                .set(wallets::balance.eq((wallets::balance + transaction.amount)))
                .execute(conn)?;

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
            .join("v2/checkout/orders/")
            .and_then(|u| u.join(order_id))
            .and_then(|u| u.join("capture"))
            .map_err(|_| ApiError::Internal("Invalid PayPal capture URL".into()))?;


        let resp = client
            .post(url)
            .bearer_auth(token)
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

        let body: PayPalCaptureResponse = resp
            .json()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Invalid PayPal capture response");
                ApiError::Payment("Invalid PayPal capture response".into())
            })?;

        let capture = body
            .purchase_units
            .get(0)
            .and_then(|pu| pu.payments.captures.get(0))
            .ok_or_else(|| ApiError::Payment("Missing PayPal capture data".into()))?;

        let currency = CurrencyCode::from_str(&capture.amount.currency_code)
            .map_err(|_| ApiError::Payment("Unsupported currency".into()))?;

        Ok(PaypalCapture {
            capture_id: capture.id.clone(),
            currency,
        })
    }
}

