use payego_primitives::error::ApiError;
use payego_primitives::models::enum_types::CurrencyCode;
use payego_primitives::models::dtos::wallet_dto::ExchangeRateResponse;
use reqwest::{Client, Url};
use std::time::Duration;

#[derive(Clone)]
pub struct ExchangeRateClient {
    http: Client,
    base_url: Url,
}

impl ExchangeRateClient {
    pub fn new(http: Client, base_url: &str) -> Result<Self, ApiError> {
        let base_url = Url::parse(base_url)
            .map_err(|_| ApiError::Internal("Invalid FX base URL".into()))?;
        Ok(Self { http, base_url })
    }

    pub async fn get_rate(&self, from: CurrencyCode, to: CurrencyCode) -> Result<f64, ApiError> {
        if from == to {
            return Ok(1.0);
        }

        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| ApiError::Internal("Invalid FX URL path".into()))?
            .push(from.to_string().as_str());

        let resp = self.http
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| ApiError::Payment(format!("FX API unreachable: {}", e)))?;

        let status = resp.status();
        let body = resp
            .json::<ExchangeRateResponse>()
            .await
            .map_err(|_| ApiError::Payment("Invalid FX response".into()))?;

        if !status.is_success() {
            return Err(ApiError::Payment(
                body.error.unwrap_or_else(|| "FX API error".into()),
            ));
        }

        body.rates
            .get(&format!("{}", to))
            .copied()
            .filter(|r| *r > 0.0)
            .ok_or_else(|| ApiError::Payment("Exchange rate not found".into()))
    }
}
