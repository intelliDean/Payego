use axum::body::Bytes;
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use hmac::KeyInit;
use hmac::{Hmac, Mac};
use payego_core::services::paystack_service::{PaystackService};
use payego_primitives::error::ApiError;
use secrecy::ExposeSecret;
use serde_json::Value;
use sha2::Sha256;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::dtos::{PaystackData, PaystackWebhook};
// https://e02e3895d11f.ngrok-free.app/webhooks/paystack




#[utoipa::path(
    post,
    path = "/webhooks/paystack",
    request_body = PaystackWebhook,
    responses(
        (status = 200, description = "Webhook processed"),
        (status = 400, description = "Invalid webhook"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Webhook"
)]
pub async fn paystack_webhook(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let signature = headers
        .get("x-paystack-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Payment("Missing Paystack signature".into()))?;

    PaystackService::verify_paystack_signature(
        state.config.paystack_details.paystack_webhook_secret.expose_secret(),
        &body,
        signature,
    )?;

    let payload: PaystackWebhook = serde_json::from_slice(&body)
        .map_err(|_| ApiError::Payment("Invalid webhook payload".into()))?;

    let mut conn = state.db.get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

    PaystackService::handle_event(&mut conn, &payload)?;

    Ok(StatusCode::OK)
}





