use diesel::ExpressionMethods;
use diesel::{Connection, RunQueryDsl};
use diesel::{OptionalExtension, QueryDsl};
use http::StatusCode;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::dtos::{
    CaptureResponse, PayPalOrderResponse, PayPalTokenResponse,
};
use payego_primitives::models::enum_types::{CurrencyCode, PaymentProvider, PaymentState};
use payego_primitives::models::transaction::Transaction;
use payego_primitives::models::wallet::Wallet;
use payego_primitives::models::wallet_ledger::NewWalletLedger;
use payego_primitives::schema::{transactions, wallet_ledger, wallets};
use secrecy::{ExposeSecret, SecretString};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::log::error;
use uuid::Uuid;

struct PaypalCapture {
    capture_id: String,
    currency: CurrencyCode,
}

#[derive(Clone)]
pub struct PayPalService;

impl PayPalService {
    async fn get_access_token(state: &AppState) -> Result<String, ApiError> {
        let resp = state
            .http_client
            .post(format!(
                "{}/v1/oauth2/token",
                state.config.paypal_details.paypal_api_url
            ))
            .basic_auth(
                &state.config.paypal_details.paypal_client_id,
                Some(state.config.paypal_details.paypal_secret.expose_secret()),
            )
            .form(&[("grant_type", "client_credentials")])
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("PayPal auth request failed: {}", e)))?;

        if resp.status() != StatusCode::OK {
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

        let resp = state
            .http_client
            .get(format!(
                "{}/v2/checkout/orders/{}",
                state.config.paypal_details.paypal_api_url, order_id
            ))
            .bearer_auth(token)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("PayPal order request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ApiError::Payment("Failed to fetch PayPal order".into()));
        }

        let order = resp
            .json::<PayPalOrderResponse>()
            .await
            .map_err(|_| ApiError::Payment("Invalid PayPal order response".into()))?;

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

        let resp = client
            .post(format!(
                "{}/v2/checkout/orders/{}/capture",
                state.config.paypal_details.paypal_api_url, order_id
            ))
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;

        let body: serde_json::Value = resp.json().await?;

        let capture = &body["purchase_units"][0]["payments"]["captures"][0];

        let capture_id = capture["id"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("Missing capture ID".into()))?
            .to_string();

        let currency_str = capture["amount"]["currency_code"]
            .as_str()
            .ok_or_else(|| ApiError::Payment("Missing currency".into()))?;

        let currency = CurrencyCode::from_str(currency_str)
            .map_err(|_| ApiError::Payment("Unsupported currency".into()))?;

        Ok(PaypalCapture {
            capture_id,
            currency,
        })
    }
}
