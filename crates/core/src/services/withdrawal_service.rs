use diesel::prelude::*;
pub use payego_primitives::{
    config::security_config::Claims,
    error::ApiError,
    models::{
        app_state::app_state::AppState,
        bank::BankAccount,
        dtos::withdrawal_dto::{WithdrawRequest, WithdrawResponse},
        enum_types::{CurrencyCode, PaymentProvider, PaymentState, TransactionIntent},
        transaction::NewTransaction,
        wallet::Wallet,
        wallet_ledger::NewWalletLedger,
    },
    schema::{bank_accounts, transactions, wallet_ledger, wallets},
};

use reqwest::{Client, Url};
use secrecy::ExposeSecret;
use serde_json::{json, Value};
use tracing::log::warn;
use uuid::Uuid;
use payego_primitives::models::providers_dto::{PaystackResponse, PaystackTransData};

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
        let wallet = wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(CurrencyCode::from(req.currency)))
            .for_update()
            .first::<Wallet>(&mut conn)?;

        if wallet.balance < amount_minor {
            return Err(ApiError::Payment("Insufficient balance".into()));
        }

        let bank_account = bank_accounts::table
            .filter(bank_accounts::id.eq(bank_account_id))
            .filter(bank_accounts::user_id.eq(user_id))
            .filter(bank_accounts::is_verified.eq(true))
            .first::<BankAccount>(&mut conn)?;

        // ---- External transfer FIRST ----
        let provider_ref = Self::initiate_paystack_transfer(
            state,
            &bank_account,
            amount_minor,
            &req.currency,
            req.reference,
        )
        .await?;

        // ---- Atomic DB write ----
        conn.transaction(|conn| {
            // Wallet update
            diesel::update(wallets::table)
                .filter(wallets::id.eq(wallet.id))
                .set(wallets::balance.eq(wallet.balance - amount_minor))
                .execute(conn)?;

            // Transaction
            let tx_id = diesel::insert_into(transactions::table)
                .values(NewTransaction {
                    user_id,
                    counterparty_id: None,
                    intent: TransactionIntent::Payout,
                    amount: amount_minor,
                    currency: wallet.currency,
                    txn_state: PaymentState::Pending,
                    provider: Some(PaymentProvider::Paystack),
                    provider_reference: Some(&provider_ref),
                    idempotency_key: &req.idempotency_key,
                    reference: req.reference,
                    description: Some("Wallet withdrawal"),
                    metadata: json!({
                        "bank_account_id": bank_account_id,
                    }),
                })
                .returning(transactions::id)
                .get_result::<Uuid>(conn)?;

            // Ledger
            diesel::insert_into(wallet_ledger::table)
                .values(NewWalletLedger {
                    wallet_id: wallet.id,
                    transaction_id: tx_id,
                    amount: amount_minor,
                })
                .execute(conn)?;

            Ok(tx_id)
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
    ) -> Result<String, ApiError> {
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
            .map_err(|_| ApiError::Payment("Failed to reach Paystack".into()))?;

        if !resp.status().is_success() {
            // warn!(status = %resp.status(), "Paystack HTTP error");
            return Err(ApiError::Payment("Paystack transfer failed".into()));
        }

        let body: PaystackResponse<PaystackTransData> = resp
            .json()
            .await
            .map_err(|_| ApiError::Payment("Invalid Paystack response".into()))?;

        if !body.status {
            // warn!(message = %body.message, "Paystack rejected transfer");
            return Err(ApiError::Payment("Transfer rejected by Paystack".into()));
        }

        let data = body.data.ok_or_else(|| {
            ApiError::Payment("Paystack response missing data".into())
        })?;

        Ok(data.transfer_code)

    }
}

