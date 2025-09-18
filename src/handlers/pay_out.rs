// use crate::config::security_config::Claims;
// use crate::models::user_models::{AppState, BankAccount, NewTransaction, Wallet};
// use axum::{extract::State, http::StatusCode, Json};
// use diesel::prelude::*;
// use reqwest::Client;
// use std::sync::Arc;
// use utoipa::ToSchema;
// use uuid::Uuid;
// 
// #[utoipa::path(
//     post,
//     path = "/api/payout",
//     request_body = PayoutRequest,
//     responses(
//         (status = 200, description = "Payout initiated"),
//         (status = 400, description = "Invalid bank or insufficient balance")
//     ),
//     security(("Bearer" = []))
// )]
// pub async fn payout(
//     State(state): State<Arc<AppState>>,
//     claims: Claims,
//     Json(req): Json<PayoutRequest>,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     let mut conn = state
//         .db
//         .get()
//         .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?;
//     let user_id = Uuid::parse_str(&claims.sub)
//         .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
//     let amount_cents = (req.amount * 100.0) as i64;
// 
//     // Validate balance and bank
//     let wallet = crate::schema::wallets::table
//         .find(user_id)
//         .first::<Wallet>(&mut conn)
//         .map_err(|_| (StatusCode::BAD_REQUEST, "Wallet not found".to_string()))?;
//     if wallet.balance < amount_cents {
//         return Err((StatusCode::BAD_REQUEST, "Insufficient balance".to_string()));
//     }
// 
//     let bank = crate::schema::bank_accounts::table
//         .find(req.bank_id)
//         .filter(crate::schema::bank_accounts::user_id.eq(user_id))
//         .first::<BankAccount>(&mut conn)
//         .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid bank account".to_string()))?;
//     if !bank.is_verified {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             "Bank account not verified".to_string(),
//         ));
//     }
// 
//     let transaction_id = Uuid::new_v4();
//     diesel::insert_into(crate::schema::transactions::table)
//         .values(NewTransaction {
//             id: transaction_id,
//             user_id,
//             recipient_id: None,
//             amount: -amount_cents,
//             transaction_type: "paystack_payout".to_string(),
//             status: "pending".to_string(),
//             provider: Some("paystack".to_string()),
//             description: Some("Bank payout".to_string()),
//             reference: Some(transaction_id.to_string()),
//             metadata: Some(serde_json::json!({ "bank_id": req.bank_id.to_string() })),
//         })
//         .execute(&mut conn)
//         .map_err(|_| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Failed to create transaction".to_string(),
//             )
//         })?;
// 
//     let client = Client::new();
//     let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "Paystack key not set".to_string(),
//         )
//     })?;
//     let resp = client
//         .post("https://api.paystack.co/transfer")
//         .header("Authorization", format!("Bearer {}", paystack_key))
//         .json(&serde_json::json!({
//             "source": "balance",
//             "amount": amount_cents,
//             "recipient": bank.paystack_recipient_code.unwrap(),
//             "reason": "Wallet payout",
//             "reference": transaction_id.to_string()
//         }))
//         .send()
//         .await
//         .map_err(|_| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Paystack API error".to_string(),
//             )
//         })?
//         .json::<serde_json::Value>()
//         .await
//         .map_err(|_| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Paystack response error".to_string(),
//             )
//         })?;
// 
//     if resp["status"].as_bool().unwrap_or(false) {
//         conn.transaction(|conn| {
//             diesel::update(crate::schema::transactions::table.find(transaction_id))
//                 .set((
//                     crate::schema::transactions::status.eq("completed"),
//                     crate::schema::transactions::updated_at.eq(chrono::Utc::now()),
//                 ))
//                 .execute(conn)?;
//             diesel::update(crate::schema::wallets::table.find(user_id))
//                 .set((
//                     crate::schema::wallets::balance
//                         .eq(crate::schema::wallets::balance - amount_cents),
//                     crate::schema::wallets::updated_at.eq(chrono::Utc::now()),
//                 ))
//                 .execute(conn)?;
//             Ok(())
//         })
//         .map_err(|_| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Failed to update transaction".to_string(),
//             )
//         })?;
//     }
// 
//     Ok(StatusCode::OK)
// }
// 
// #[derive(ToSchema)]
// pub struct PayoutRequest {
//     amount: f64, // In dollars
//     bank_id: Uuid,
// }
