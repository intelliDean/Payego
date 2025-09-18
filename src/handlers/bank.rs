// use crate::config::security_config::Claims;
// use crate::models::user_models::{AppState, NewBankAccount};
// use axum::{Json, extract::State, http::StatusCode};
// use diesel::prelude::*;
// use reqwest::Client;
// use serde::Deserialize;
// use std::sync::Arc;
// use utoipa::ToSchema;
// use uuid::Uuid;
// 
// #[utoipa::path(
//     post,
//     path = "/api/bank",
//     request_body = BankRequest,
//     responses(
//         (status = 201, description = "Bank account added"),
//         (status = 400, description = "Invalid bank details")
//     ),
//     security(("Bearer" = []))
// )]
// pub async fn add_bank(
//     State(state): State<Arc<AppState>>,
//     claims: Claims,
//     Json(req): Json<BankRequest>,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     let conn = &mut state
//         .db
//         .get()
//         .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?;
//     let user_id = Uuid::parse_str(&claims.sub)
//         .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
//     let client = Client::new();
//     let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "Paystack key not set".to_string(),
//         )
//     })?;
// 
//     // Verify bank account with Paystack
//     let resp = client
//         .post("https://api.paystack.co/transferrecipient")
//         .header("Authorization", format!("Bearer {}", paystack_key))
//         .json(&serde_json::json!({
//             "type": "nuban",
//             "name": req.account_name.unwrap_or("User Bank".to_string()),
//             "account_number": req.account_number,
//             "bank_code": req.bank_code,
//             "currency": "NGN"
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
//     let recipient_code = resp["data"]["recipient_code"]
//         .as_str()
//         .ok_or((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "Invalid Paystack response".to_string(),
//         ))?
//         .to_string();
// 
//     diesel::insert_into(crate::schema::bank_accounts::table)
//         .values(NewBankAccount {
//             id: Uuid::new_v4(),
//             user_id,
//             bank_code: req.bank_code,
//             account_number: req.account_number,
//             account_name: req.account_name,
//             paystack_recipient_code: Some(recipient_code),
//             is_verified: true,
//         })
//         .execute(conn)
//         .map_err(|_| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Failed to add bank account".to_string(),
//             )
//         })?;
// 
//     Ok(StatusCode::CREATED)
// }
// 
// #[derive(Deserialize, ToSchema)]
// pub struct BankRequest {
//     bank_code: String,
//     account_number: String,
//     account_name: Option<String>,
// }
