use crate::client::{CreateTransferRecipientRequest, PaystackClient};
use diesel::prelude::*;
use tracing::{error, warn};
use uuid::Uuid;

use crate::services::bank_service::BankService;
use payego_primitives::models::enum_types::CurrencyCode;
pub use payego_primitives::{
    config::security_config::Claims,
    error::{ApiError, AuthError},
    models::{app_state::app_state::AppState, bank::BankAccount},
    models::{
        bank::NewBankAccount,
        dtos::bank_dtos::{
            BankAccountResponse, BankAccountsResponse, BankRequest, PaystackRecipientResponse,
        },
    },
    schema::bank_accounts,
};

pub struct BankAccountService;

impl BankAccountService {
    pub async fn list_user_accounts(
        state: &AppState,
        claims: &Claims,
    ) -> Result<BankAccountsResponse, ApiError> {
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            warn!("bank_accounts.list: invalid subject in token");
            ApiError::Auth(AuthError::InvalidToken("Invalid token".into()))
        })?;

        let mut conn = state.db.get().map_err(|_| {
            error!("bank_accounts.list: failed to acquire db connection");
            ApiError::DatabaseConnection("Database unavailable".into())
        })?;

        let accounts = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id))
            .order(bank_accounts::created_at.desc())
            .load::<BankAccount>(&mut conn)
            .map_err(|_| {
                error!("bank_accounts.list: query failed");
                ApiError::Internal("Failed to fetch bank accounts".into())
            })?;

        Ok(BankAccountsResponse {
            bank_accounts: accounts
                .into_iter()
                .map(BankAccountResponse::from)
                .collect(),
        })
    }

    pub async fn get_bank_accounts(
        state: &AppState,
        user_id_val: Uuid,
    ) -> Result<Vec<BankAccount>, ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e: r2d2::Error| ApiError::DatabaseConnection(e.to_string()))?;

        let accounts = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id_val))
            .load::<BankAccount>(&mut conn)
            .map_err(ApiError::from)?;

        Ok(accounts)
    }

    pub async fn create_bank_account(
        state: &AppState,
        user_id_val: Uuid,
        req: BankRequest,
    ) -> Result<BankAccount, ApiError> {
        let mut conn = state.db.get().map_err(|e: r2d2::Error| {
            error!("Database error: {}", e);
            ApiError::DatabaseConnection(e.to_string())
        })?;

        // Idempotency check
        if let Ok(existing) = bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id_val))
            .filter(bank_accounts::bank_code.eq(&req.bank_code))
            .filter(bank_accounts::account_number.eq(&req.account_number))
            .first::<BankAccount>(&mut conn)
        {
            return Ok(existing);
        }

        let account_details =
            BankService::resolve_account_details(state, &req.bank_code, &req.account_number)
                .await?;

        let account_name = account_details.account_name;

        //make call to paystack via its client
        let paystack_client = PaystackClient::new(
            state.http_client.clone(),
            &state.config.paystack_details.paystack_api_url,
            state.config.paystack_details.paystack_secret_key.clone(),
        )?;

        let payload = PaystackClient::create_recipient(
            &*account_name,
            &req.account_number,
            &req.bank_code,
            CurrencyCode::NGN,
        );

        let recipient_code = paystack_client
            .create_transfer_recipient(payload)
            .await
            .map_err(|_| ApiError::Payment("Unable to create transfer recipient".into()))?;

        let new_account = NewBankAccount {
            user_id: user_id_val,
            bank_name: Some(&req.bank_name),
            account_number: &req.account_number,
            account_name: Some(&*account_name),
            bank_code: &*req.bank_code,
            provider_recipient_id: Some(&*recipient_code),
            is_verified: true, // rename later
        };

        let account = diesel::insert_into(bank_accounts::table)
            .values(&new_account)
            .get_result(&mut conn)?;

        Ok(account)
    }
}

// let url = Self::paystack_url(
// &state.config.paystack_details.paystack_api_url,
// "transferrecipient",
// )?;
//
// let payload = CreateRecipientRequest {
// recipient_type: "nuban",
// name: &*account_name,
// account_number: &req.account_number,
// bank_code: &req.bank_code,
// currency: CurrencyCode::NGN,
// };
//
// let resp = state
// .http_client
// .post(url)
// .bearer_auth(
// state
// .config
// .paystack_details
// .paystack_secret_key
// .expose_secret(),
// )
// .json(&payload)
// .send()
// .await
// .map_err(|_| ApiError::Payment("Paystack recipient creation failed".into()))?;
//
// let body: PaystackRecipientResponse = resp
// .json()
// .await
// .map_err(|_| ApiError::Payment("Invalid Paystack response".into()))?;
