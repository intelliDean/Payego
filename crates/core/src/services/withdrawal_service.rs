use crate::repositories::bank_account_repository::BankAccountRepository;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::wallet_repository::WalletRepository;
use diesel::prelude::*;
pub use payego_primitives::{
    config::security_config::Claims,
    error::ApiError,
    models::{
        app_state::AppState,
        bank::BankAccount,
        dtos::withdrawal_dto::{WithdrawRequest, WithdrawResponse},
        enum_types::{CurrencyCode, PaymentProvider, PaymentState, TransactionIntent},
        transaction::NewTransaction,
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
    schema::{bank_accounts, transactions, wallet_ledger, wallets},
};

use payego_primitives::models::providers_dto::{PaystackResponse, PaystackTransData};
use reqwest::Url;
use tracing::error;
use secrecy::ExposeSecret;
use serde_json::json;
use uuid::Uuid;

pub struct WithdrawalService;

impl WithdrawalService {
    pub async fn withdraw(
        state: &AppState,
        user_id: Uuid,
        bank_account_id: Uuid,
        req: WithdrawRequest,
    ) -> Result<WithdrawResponse, ApiError> {
        let amount_minor = Self::convert_to_minor_units(req.amount)?;

        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        // ---- Load wallet (FOR UPDATE) ----
        let wallet = WalletRepository::find_by_user_and_currency_with_lock(&mut conn, user_id, req.currency)?;

        if wallet.balance < amount_minor {
            return Err(ApiError::Payment("Insufficient balance".into()));
        }

        let bank_account = BankAccountRepository::find_verified_by_id_and_user(&mut conn, bank_account_id, user_id)?;

        // ---- External transfer FIRST ----
        let provider_data = Self::initiate_paystack_transfer(
            state,
            &bank_account,
            amount_minor,
            &req.currency,
            req.reference,
        )
        .await?;

        let initial_state = match provider_data.status.as_deref() {
            Some("success") => PaymentState::Completed,
            _ => PaymentState::Pending,
        };

        // ---- Atomic DB write ----
        conn.transaction(|conn| {
            // Wallet update
            WalletRepository::debit(conn, wallet.id, amount_minor)?;

            // Transaction
            let tx = TransactionRepository::create(conn, NewTransaction {
                user_id,
                counterparty_id: None,
                intent: TransactionIntent::Payout,
                amount: amount_minor,
                currency: wallet.currency,
                txn_state: initial_state,
                provider: Some(PaymentProvider::Paystack),
                provider_reference: Some(&provider_data.transfer_code),
                idempotency_key: &req.idempotency_key,
                reference: req.reference,
                description: Some("Wallet withdrawal"),
                metadata: json!({
                    "bank_account_id": bank_account_id,
                }),
            })?;

            // Ledger
            WalletRepository::add_ledger_entry(conn, NewWalletLedger {
                wallet_id: wallet.id,
                transaction_id: tx.id,
                amount: -amount_minor,
            })?;

            Ok(tx.id)
        })
        .map(|tx_id| WithdrawResponse {
            transaction_id: tx_id,
        })
    }

    fn convert_to_minor_units(amount: f64) -> Result<i64, ApiError> {
        if amount <= 0.0 {
            return Err(ApiError::Internal("Invalid amount".into()));
        }

        Ok((amount * 100.0).round() as i64)
    }

    async fn initiate_paystack_transfer(
        state: &AppState,
        bank: &BankAccount,
        amount_minor: i64,
        currency: &CurrencyCode,
        reference: Uuid,
    ) -> Result<PaystackTransData, ApiError> {
        let key = state
            .config
            .paystack_details
            .paystack_secret_key
            .expose_secret();

        let mut url = Url::parse(&state.config.paystack_details.paystack_api_url)
            .map_err(|_| ApiError::Internal("Invalid Paystack base URL".into()))?;

        url.set_path("transfer");

        let resp = state
            .http_client
            .post(url)
            .bearer_auth(key)
            .json(&json!({
                "source": "balance",
                "amount": amount_minor,
                "recipient": bank.provider_recipient_id,
                "reference": reference.to_string(),
                "reason": format!("Withdrawal ({})", currency),
            }))
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to reach Paystack during transfer initiation");
            ApiError::Payment(format!("Failed to reach Paystack: {}", e))
        })?;

        let status = resp.status();
        let body_text = resp.text().await.map_err(|e| {
            error!(error = %e, "Failed to read Paystack transfer response body");
            ApiError::Payment("Invalid Paystack response body".into())
        })?;

        if !status.is_success() {
            error!(
                status = %status,
                body = %body_text,
                "Paystack transfer initiation failed"
            );
            return Err(ApiError::Payment(format!("Paystack transfer failed with status {}: {}", status, body_text)));
        }

        let body: PaystackResponse<PaystackTransData> = serde_json::from_str(&body_text)
            .map_err(|e| {
                error!(error = %e, body = %body_text, "Failed to parse Paystack transfer response");
                ApiError::Payment("Invalid Paystack response format".into())
            })?;

        if !body.status {
            // warn!(message = %body.message, "Paystack rejected transfer");
            return Err(ApiError::Payment("Transfer rejected by Paystack".into()));
        }

        body.data
            .ok_or_else(|| ApiError::Payment("Paystack response missing data".into()))
    }
}
