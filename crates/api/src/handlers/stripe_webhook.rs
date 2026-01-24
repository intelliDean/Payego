use payego_primitives::error::ApiErrorResponse;
use axum::body::Bytes;
use axum::extract::State;
use http::{HeaderMap, StatusCode};
use payego_core::services::stripe_service::{ApiError, AppState, StripeService};
use std::sync::Arc;
use tracing::info;
use payego_core::services::stripe_service::WebhookOutcome;

#[utoipa::path(
    post,
    path = "/api/webhooks/stripe",
    tag = "Webhooks",
    summary = "Receive and process Stripe webhook events",
    description = "Public endpoint that receives asynchronous event notifications from Stripe \
                   (e.g. `payment_intent.succeeded`, `charge.refunded`, `invoice.payment_succeeded`, \
                   `customer.subscription.created`, etc.). \
                   The server **must** verify the request signature using the `Stripe-Signature` header \
                   and your Stripe webhook signing secret before processing the payload. \
                   Always respond with HTTP **200 OK** as quickly as possible — even if internal processing fails — \
                   to prevent Stripe from retrying the event (Stripe retries aggressively on non-2xx responses). \
                   This is a **public endpoint** (no authentication required). \
                   Stripe may send duplicate events; your handler **must be idempotent** (use event `id` as idempotency key). \
                   Events are sent as JSON with `id`, `type`, `data.object`, and other metadata.",
    operation_id = "receiveStripeWebhook",
    request_body(
        content = String,
        description = "Raw JSON payload of the Stripe event. \
                       **Do not** parse this body directly in the schema — verify signature first using the raw bytes/string.",
    ),
    responses(
        ( status = 200, description = "Webhook successfully received and acknowledged. \
                           Stripe requires 200 OK — do **not** return 4xx or 5xx even if processing fails later."),
        ( status = 400, description = "Bad request — invalid payload format, failed signature verification, \
                           missing required headers, or malformed JSON", body = ApiErrorResponse),
        ( status = 422, description = "Unprocessable entity — valid signature but event structure invalid \
                           (missing `id`, `type`, or expected fields in `data.object`)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests — rate limit exceeded (protects your server from flood)", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error — should be avoided. \
                           Stripe will retry on 5xx responses (up to 3 days with exponential backoff).", body = ApiErrorResponse),
    ),
    security(()),
)]



pub async fn stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    info!("Stripe webhook received");

    let event = StripeService::construct_event(&state, headers, &body)?;

    match StripeService::handle_event(&state, event)? {
        WebhookOutcome::Processed => {
            info!("Stripe webhook processed");
            Ok(StatusCode::OK)
        }
        WebhookOutcome::Ignored => {
            info!("Stripe webhook ignored");
            Ok(StatusCode::OK)
        }
    }
}







// use payego_primitives::models::AppState;
// use payego_primitives::error::ApiError;
// use crate::handlers::paypal_capture::Transaction;
// use payego_primitives::schema::{transactions, wallets};
// use axum::extract::State;
// use diesel::prelude::*;
// use http::{HeaderMap, StatusCode};
// use serde::Deserialize;
// use std::sync::Arc;
// use stripe::{Currency, Event, EventObject, PaymentIntentStatus, Webhook};
// use tracing::log::debug;
// use tracing::{error, info};
// use uuid::Uuid;
//
//
// #[utoipa::path(
//     post,
//     path = "/api/webhook/stripe",
//     request_body = String,
//     responses(
//         (status = 200, description = "Webhook received"),
//         (status = 400, description = "Invalid webhook payload or signature"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Transaction"
// )]
// pub async fn stripe_webhook(
//     State(state): State<Arc<AppState>>,
//     headers: HeaderMap,
//     payload: String,
// ) -> Result<StatusCode, ApiError> {
//     info!("Webhook called with payload length: {}", payload.len());
//
//     let signature = headers
//         .get("stripe-signature")
//         .ok_or((
//             StatusCode::BAD_REQUEST,
//             "Missing stripe-signature".to_string(),
//         ))?
//         .to_str()
//         .map_err(|_| {
//             (
//                 StatusCode::BAD_REQUEST,
//                 "Invalid stripe-signature header".to_string(),
//             )
//         })?;
//
//     let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").map_err(|_| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "STRIPE_WEBHOOK_SECRET not set".to_string(),
//         )
//     })?;
//
//     let sent_event: Event = Webhook::construct_event(&payload, signature, &webhook_secret)
//         .map_err(|e| {
//             error!("Webhook validation failed: {}", e);
//             ApiError::Webhook(e)
//         })?;
//
//     info!(
//         "Event parsed successfully: type={}, id={}",
//         sent_event.type_, sent_event.id
//     );
//
//     let conn = &mut state
//         .db
//         .get()
//         .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
//
//     match sent_event.data.object {
//         EventObject::PaymentIntent(payment_intent) => {
//             let transaction_id_str = payment_intent.metadata
//                 .get("transaction_id")
//                 .ok_or((
//                 StatusCode::BAD_REQUEST,
//                 "Missing transaction_id in metadata".to_string(),
//             ))?;
//
//             let transaction_id = Uuid::parse_str(transaction_id_str).map_err(|e| {
//                 (
//                     StatusCode::BAD_REQUEST,
//                     format!("Invalid transaction_id: {}", e),
//                 )
//             })?;
//
//             info!("Checking Idempotency");
//             // Check for idempotency
//             let existing = transactions::table
//                 .filter(transactions::reference.eq(transaction_id))
//                 .filter(transactions::status.eq("completed"))
//                 .select(diesel::dsl::count_star())
//                 .first::<i64>(conn)
//                 .map(|count| count > 0)
//                 .map_err(|e| {
//                     error!("Transaction update failed: {}", e);
//                     if e.to_string().contains("not found") {
//                         ApiError::Payment("Transaction not found".to_string())
//                     } else {
//                         ApiError::from(e)
//                     }
//                 })?;
//
//             if existing {
//                 info!("Transaction {} already processed", transaction_id);
//                 return Ok(StatusCode::OK);
//             }
//
//             // Validate currency
//             let transaction = transactions::table
//                 .filter(transactions::reference.eq(transaction_id))
//                 .select(Transaction::as_select())
//                 .first(conn)
//                 .map_err(|e| {
//                     error!("Failed to fetch transaction: {}", e);
//                     ApiError::Payment("Transaction not found".to_string())
//                 })?;
//
//             let payment_intent_currency = payment_intent.currency.to_string().to_uppercase();
//
//             if transaction.currency != payment_intent_currency {
//                 error!(
//                     "Currency mismatch: transaction currency {}, payment intent currency {}",
//                     transaction.currency, payment_intent_currency
//                 );
//                 return Err((
//                     StatusCode::BAD_REQUEST,
//                     "Currency mismatch".to_string(),
//                 ));
//             }
//
//
//             match payment_intent.status {
//                 PaymentIntentStatus::Succeeded => {
//                     // Update transactions and wallets atomically
//                     conn.transaction(|conn| {
//                         info!("Updating transaction");
//                         // Update transactions table
//                         let updated_transaction = diesel::update(transactions::table)
//                             .filter(transactions::reference.eq(transaction_id))
//                             .set((
//                                 transactions::status.eq("completed"),
//                                 transactions::description.eq("Stripe top-up completed"),
//                             ))
//                             .returning(Transaction::as_select())
//                             .get_result(conn)
//                             .map_err(|e| {
//                                 error!("Transaction update failed: {}", e);
//                                 if e.to_string().contains("not found") {
//                                     ApiError::Payment("Transaction not found".to_string())
//                                 } else {
//                                     ApiError::from(e)
//                                 }
//                             })?;
//
//                         info!("Updating wallet for user {} in {}", updated_transaction.user_id, payment_intent.currency);
//                         // Update wallets table
//                         diesel::insert_into(wallets::table)
//                             .values((
//                                 wallets::user_id.eq(updated_transaction.user_id),
//                                 wallets::balance.eq(updated_transaction.amount),
//                                 wallets::currency.eq(payment_intent.currency.to_string().to_uppercase()),
//                             ))
//                             .on_conflict((wallets::user_id, wallets::currency))
//                             .do_update()
//                             .set(wallets::balance.eq(diesel::dsl::sql::<diesel::sql_types::BigInt>("balance + ").bind::<diesel::sql_types::BigInt, _>(updated_transaction.amount)))
//                             .execute(conn)
//                             .map_err(|e| {
//                                 error!("Transaction update failed: {}", e);
//                                 if e.to_string().contains("not found") {
//                                     ApiError::Payment("Transaction not found".to_string())
//                                 } else {
//                                     ApiError::from(e)
//                                 }
//                             })?;
//
//                         Ok::<(), ApiError>(())
//                     })?;
//
//                     info!(
//                         "Stripe payment succeeded for transaction: {}, currency: {}",
//                         transaction_id, payment_intent.currency
//                     );
//                 }
//                 PaymentIntentStatus::Canceled => {
//                     let description = format!(
//                         "Stripe top-up failed: {}",
//                         payment_intent
//                             .last_payment_error
//                             .map(|e| e.message.unwrap_or("Unknown error".to_string()))
//                             .unwrap_or("No error details".to_string())
//                     );
//
//                     diesel::update(transactions::table)
//                         .filter(transactions::reference.eq(transaction_id))
//                         .set((
//                             transactions::status.eq("failed"),
//                             transactions::description.eq(description),
//                         ))
//                         .execute(conn)
//                         .map_err(|e| {
//                             error!("Transaction update failed: {}", e);
//                             if e.to_string().contains("not found") {
//                                 ApiError::Payment("Transaction not found".to_string())
//                             } else {
//                                 ApiError::from(e)
//                             }
//                         })?;
//
//                     info!(
//                         "Stripe payment failed/canceled for transaction: {}",
//                         transaction_id
//                     );
//                 }
//                 _ => {
//                     info!(
//                         "Unhandled payment intent status: {:?}",
//                         payment_intent.status
//                     );
//                 }
//             }
//         }
//         _ => {
//             info!("Received unhandled Stripe event: {}", sent_event.type_);
//         }
//     }
//
//     Ok(StatusCode::OK)
// }

//===
