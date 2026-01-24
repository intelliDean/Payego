use payego_primitives::error::ApiErrorResponse;
use axum::{extract::Json, extract::State, Extension};
use payego_core::services::payment_service::{
    ApiError, AppState, Claims, PaymentService, TopUpRequest, TopUpResponse,
};
use std::sync::Arc;
use diesel::RunQueryDsl;
use http::header::{CONTENT_TYPE, USER_AGENT};
use reqwest::Url;
use secrecy::ExposeSecret;
use serde_json::json;
use tracing::log::error;
use uuid::Uuid;
use validator::Validate;
use payego_primitives::models::enum_types::{PaymentProvider, PaymentState, TransactionIntent};
use payego_primitives::models::providers_dto::PayPalOrderResp;
use payego_primitives::models::transaction::{NewTransaction, Transaction};
use payego_primitives::schema::transactions;

#[utoipa::path(
    post,
    path = "/api/wallet/top_up",
    tag = "Wallet",
    summary = "Initiate wallet top-up (deposit)",
    description = "Starts a wallet funding process by creating a payment session or authorization with the chosen payment provider. \
                   Depending on the selected method, this may return a payment URL (redirect), client_secret (for 3DS/confirmation), \
                   or transaction reference for polling. \
                   The operation is **idempotent** when an `Idempotency-Key` header is provided — retries with the same key return the original session/response. \
                   Amount must be positive and within allowed limits per currency/provider. \
                   After successful payment confirmation (via webhook or polling), the wallet balance is credited.",
    operation_id = "initiateTopUp",
    request_body(
        content = TopUpRequest,
        description = "Top-up details: amount, currency, payment method/provider, optional payment channel (card, bank transfer, mobile money, etc.), \
                       and metadata (e.g. return URL, reference)",
    ),
    responses(
        ( status = 200, description = "Top-up session successfully initiated (or idempotent retry). \
                           Returns payment URL/reference, client secret, or polling instructions depending on provider.", body = TopUpResponse),
        ( status = 400, description = "Bad request — invalid input (negative amount, unsupported currency, invalid channel, missing fields)", body = ApiErrorResponse),
        ( status = 401, description = "Unauthorized — missing or invalid authentication token", body = ApiErrorResponse),
        ( status = 402, description = "Payment required — insufficient funds or payment method declined (pre-check failure)", body = ApiErrorResponse),
        ( status = 409, description = "Conflict — duplicate top-up detected via idempotency key (returns original session if already initiated)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded on top-up initiations", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — failed to initiate payment session", body = ApiErrorResponse),
        ( status = 502, description = "Bad gateway — payment provider (Paystack/Stripe/etc.) returned an error or is unavailable", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = []))
)]
pub async fn top_up(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    payload: Result<Json<TopUpRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<TopUpResponse>, ApiError> {
    let Json(req) = payload.map_err(|rejection| {
        error!("JSON rejection: {}", rejection);
        ApiError::Payment(format!("Invalid JSON payload: {}", rejection))
    })?;

    req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let user_id = claims.user_id()?;

    Ok(Json(
        PaymentService::initiate_top_up(&state, user_id, req).await?,
    ))
}
