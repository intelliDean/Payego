// use std::sync::Arc;
// use axum::{extract::State, http::StatusCode, Json};
// use diesel::prelude::*;
// use serde_json::Value;
// use crate::models::user_models::AppState;
// 
// #[utoipa::path(
//     post,
//     path = "/webhooks/stripe",
//     request_body = Value,
//     responses(
//         (status = 200, description = "Webhook processed"),
//         (status = 400, description = "Invalid webhook")
//     )
// )]
// pub async fn stripe_webhook(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<Value>,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     let event_type = payload["type"].as_str().ok_or((StatusCode::BAD_REQUEST, "Invalid event type".to_string()))?;
//     if event_type == "payment_intent.succeeded" {
//         let transaction_id = payload["data"]["object"]["metadata"]["transaction_id"]
//             .as_str()
//             .ok_or((StatusCode::BAD_REQUEST, "Missing transaction ID".to_string()))?;
//         let transaction_id = uuid::Uuid::parse_str(transaction_id)
//             .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid transaction ID".to_string()))?;
//         let amount = payload["data"]["object"]["amount"]
//             .as_i64()
//             .ok_or((StatusCode::BAD_REQUEST, "Invalid amount".to_string()))?;
// 
//         let mut conn = state.db.get().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?;
//         conn.transaction(|conn| {
//             diesel::update(crate::schema::transactions::table.find(transaction_id))
//                 .set((
//                     crate::schema::transactions::status.eq("completed"),
//                     crate::schema::transactions::updated_at.eq(chrono::Utc::now()),
//                 ))
//                 .execute(conn)?;
//             let user_id = crate::schema::transactions::table
//                 .find(transaction_id)
//                 .select(crate::schema::transactions::user_id)
//                 .first::<uuid::Uuid>(conn)?;
//             diesel::update(crate::schema::wallets::table.find(user_id))
//                 .set((
//                     crate::schema::wallets::balance.eq(crate::schema::wallets::balance + amount),
//                     crate::schema::wallets::updated_at.eq(chrono::Utc::now()),
//                 ))
//                 .execute(conn)?;
//             Ok(())
//         })
//             .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process webhook".to_string()))?;
//     }
// 
//     Ok(StatusCode::OK)
// }
// 
// #[utoipa::path(
//     post,
//     path = "/webhooks/paystack",
//     request_body = Value,
//     responses(
//         (status = 200, description = "Webhook processed"),
//         (status = 400, description = "Invalid webhook")
//     )
// )]
// pub async fn paystack_webhook(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<Value>,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     let event = payload["event"].as_str().ok_or((StatusCode::BAD_REQUEST, "Invalid event".to_string()))?;
//     if event == "transfer.success" {
//         let transaction_id = payload["data"]["reference"]
//             .as_str()
//             .ok_or((StatusCode::BAD_REQUEST, "Missing reference".to_string()))?;
//         let transaction_id = uuid::Uuid::parse_str(transaction_id)
//             .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid transaction ID".to_string()))?;
// 
//         let mut conn = state.db.get().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?;
//         diesel::update(crate::schema::transactions::table.find(transaction_id))
//             .set((
//                 crate::schema::transactions::status.eq("completed"),
//                 crate::schema::transactions::updated_at.eq(chrono::Utc::now()),
//             ))
//             .execute(&mut conn)
//             .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process webhook".to_string()))?;
//     }
// 
//     Ok(StatusCode::OK)
// }