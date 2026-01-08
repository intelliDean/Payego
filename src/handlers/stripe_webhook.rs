// use crate::AppState;
// use crate::error::ApiError;
// use crate::handlers::paypal_capture::Transaction;
// use crate::schema::{transactions, wallets};
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
// ) -> Result<StatusCode, (StatusCode, String)> {
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
//                         ApiError::Database(e)
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
//                                     ApiError::Database(e)
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
//                             .set(wallets::balance.eq(wallets::balance + updated_transaction.amount))
//                             .execute(conn)
//                             .map_err(|e| {
//                                 error!("Transaction update failed: {}", e);
//                                 if e.to_string().contains("not found") {
//                                     ApiError::Payment("Transaction not found".to_string())
//                                 } else {
//                                     ApiError::Database(e)
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
//                                 ApiError::Database(e)
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


use crate::AppState;
use crate::error::ApiError;
use crate::handlers::paypal_capture::Transaction;
use crate::schema::{transactions, wallets};
use axum::extract::State;
use diesel::prelude::*;
use http::{HeaderMap, StatusCode};
use std::sync::Arc;
use stripe::{Event, EventObject, EventType, Webhook};
use tracing::{error, info};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/webhook/stripe",
    request_body = String,
    responses(
        (status = 200, description = "Webhook received"),
        (status = 400, description = "Invalid webhook payload or signature"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Transaction"
)]
pub async fn stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: String,
) -> Result<StatusCode, (StatusCode, String)> {
    info!("Webhook called with payload length: {}", payload.len());

    let signature = headers
        .get("stripe-signature")
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Missing stripe-signature".to_string(),
        ))?
        .to_str()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid stripe-signature header".to_string(),
            )
        })?;

    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "STRIPE_WEBHOOK_SECRET not set".to_string(),
        )
    })?;

    let sent_event: Event = Webhook::construct_event(&payload, signature, &webhook_secret)
        .map_err(|e| {
            error!("Webhook validation failed: {}", e);
            ApiError::Webhook(e)
        })?;

    info!(
        "Event parsed successfully: type={}, id={}",
        sent_event.type_, sent_event.id
    );

    let conn = &mut state
        .db
        .get()
        .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

    match sent_event.type_ {
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(session) = sent_event.data.object {
                let transaction_id_str = session
                    .metadata
                    .as_ref()
                    .ok_or(ApiError::Auth("Missing metadata in session".to_string()))?
                    .get("transaction_id")
                    .ok_or(ApiError::Auth("Missing transaction_id in metadata".to_string()))?;

                let transaction_id = Uuid::parse_str(transaction_id_str).map_err(|e| {
                    error!("Invalid transaction_id: {}", e);
                    ApiError::Auth(format!("Invalid transaction_id: {}", e))
                })?;

                info!("Processing checkout.session.completed for transaction {}", transaction_id);

                // Check for idempotency
                let existing = transactions::table
                    .filter(transactions::reference.eq(transaction_id))
                    .filter(transactions::status.eq("completed"))
                    .select(diesel::dsl::count_star())
                    .first::<i64>(conn)
                    .map(|count| count > 0)
                    .map_err(|e| {
                        error!("Idempotency check failed: {}", e);
                        if e.to_string().contains("not found") {
                            ApiError::Payment("Transaction not found".to_string())
                        } else {
                            ApiError::Database(e)
                        }
                    })?;

                if existing {
                    info!("Transaction {} already processed", transaction_id);
                    return Ok(StatusCode::OK);
                }

                // Fetch transaction
                let transaction = transactions::table
                    .filter(transactions::reference.eq(transaction_id))
                    .select(Transaction::as_select())
                    .first(conn)
                    .map_err(|e| {
                        error!("Failed to fetch transaction: {}", e);
                        ApiError::Payment("Transaction not found".to_string())
                    })?;

                // Validate currency
                let payment_currency = session.currency.as_ref().map(|c| c.to_string().to_uppercase());
                if let Some(curr) = payment_currency {
                    if transaction.currency != curr {
                        error!(
                            "Currency mismatch: transaction currency {}, session currency {}",
                            transaction.currency, curr
                        );
                        return Err((
                            StatusCode::BAD_REQUEST,
                            "Currency mismatch".to_string(),
                        ));
                    }
                }

                // Update transaction and wallet atomically
                conn.transaction(|conn| {
                    info!("Updating transaction {}", transaction_id);
                    let updated_transaction = diesel::update(transactions::table)
                        .filter(transactions::reference.eq(transaction_id))
                        .set((
                            transactions::status.eq("completed"),
                            transactions::description.eq("Stripe top-up completed"),
                            transactions::updated_at.eq(chrono::Utc::now()),
                        ))
                        .returning(Transaction::as_select())
                        .get_result(conn)
                        .map_err(|e| {
                            error!("Transaction update failed: {}", e);
                            if e.to_string().contains("not found") {
                                ApiError::Payment("Transaction not found".to_string())
                            } else {
                                ApiError::Database(e)
                            }
                        })?;

                    info!("Updating wallet for user {} in {}", updated_transaction.user_id, updated_transaction.currency);
                    diesel::insert_into(wallets::table)
                        .values((
                            wallets::user_id.eq(updated_transaction.user_id),
                            wallets::balance.eq(updated_transaction.amount),
                            wallets::currency.eq(updated_transaction.currency),
                        ))
                        .on_conflict((wallets::user_id, wallets::currency))
                        .do_update()
                        .set(wallets::balance.eq(wallets::balance + updated_transaction.amount))
                        .execute(conn)
                        .map_err(|e| {
                            error!("Wallet update failed: {}", e);
                            if e.to_string().contains("not found") {
                                ApiError::Payment("Wallet not found".to_string())
                            } else {
                                ApiError::Database(e)
                            }
                        })?;

                    Ok::<(), ApiError>(())
                })?;

                info!("Stripe payment succeeded for transaction: {}", transaction_id);
            } else {
                info!("Unexpected event object for checkout.session.completed");
            }
        }
        _ => {
            info!("Received unhandled Stripe event: {}", sent_event.type_);
        }
    }

    Ok(StatusCode::OK)
}