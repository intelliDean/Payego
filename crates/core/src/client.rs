use payego_primitives::error::{ApiError, PaystackError};
use payego_primitives::models::bank_dtos::PaystackResolveResponse;
pub use payego_primitives::models::clients_dto::{
    CreateTransferRecipientRequest, CreateTransferRecipientResponse,
};
use payego_primitives::models::enum_types::CurrencyCode;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};
use tracing::warn;

#[derive(Clone)]
pub struct PaystackClient {
    http: Client,
    base_url: Url,
    secret_key: SecretString,
}

impl PaystackClient {
    pub fn new(http: Client, base_url: &str, secret_key: SecretString) -> Result<Self, ApiError> {
        let base_url = Url::parse(base_url)
            .map_err(|_| ApiError::Internal("Invalid Paystack base URL".into()))?;

        Ok(Self {
            http,
            base_url,
            secret_key,
        })
    }

    pub fn create_recipient<'a>(
        name: &'a str,
        account_number: &'a str,
        bank_code: &'a str,
        currency: CurrencyCode,
    ) -> CreateTransferRecipientRequest<'a> {
        CreateTransferRecipientRequest {
            recipient_type: "nuban",
            name,
            account_number,
            bank_code,
            currency,
        }
    }

    pub async fn create_transfer_recipient(
        &self,
        payload: CreateTransferRecipientRequest<'_>,
    ) -> Result<String, PaystackError> {
        let url = self.endpoint("transferrecipient");

        let resp = self
            .http
            .post(url)
            .bearer_auth(self.secret_key.expose_secret())
            .json(&payload)
            .send()
            .await
            .map_err(|_| PaystackError::RequestFailed)?;

        let status = resp.status();

        let body: CreateTransferRecipientResponse = resp
            .json()
            .await
            .map_err(|_| PaystackError::RequestFailed)?;

        if !status.is_success() || !body.status {
            warn!(
                paystack_message = %body.message,
                "Paystack create_transfer_recipient failed"
            );
            return Err(PaystackError::Api(body.message));
        }

        body.data
            .map(|d| d.recipient_code)
            .ok_or(PaystackError::Api("Missing recipient_code".into()))
    }

    pub async fn resolve_bank_account(
        &self,
        account_number: &str,
        bank_code: &str,
    ) -> Result<PaystackResolveResponse, ApiError> {
        let mut url = self.base_url.clone();

        url.set_path("bank/resolve");
        url.query_pairs_mut()
            .append_pair("account_number", account_number)
            .append_pair("bank_code", bank_code);

        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.secret_key.expose_secret())
            .header("User-Agent", "Payego/1.0")
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to reach Paystack");
                ApiError::Payment("Paystack service unavailable".into())
            })?;

        let status = resp.status();

        let body_text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            tracing::warn!(
                http_status = status.as_u16(),
                response = %body_text.chars().take(200).collect::<String>(),
                "Paystack bank resolve failed"
            );
            return Err(ApiError::Payment("Paystack request failed".into()));
        }

        let body: PaystackResolveResponse = serde_json::from_str(&body_text).map_err(|e| {
            tracing::error!(
                error = %e,
                response = %body_text.chars().take(200).collect::<String>(),
                "Invalid JSON from Paystack"
            );
            ApiError::Payment("Invalid Paystack response".into())
        })?;

        if !body.status {
            tracing::warn!(
                message = %body.message,
                "Paystack rejected bank resolution"
            );
            return Err(ApiError::Payment(body.message));
        }

        Ok(body)
    }

    fn endpoint(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(path);
        url
    }
}
