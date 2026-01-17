// use axum::extract::State;
// use diesel::prelude::*;
// use http::StatusCode;
// use payego_core::services::bank_service::BankService;
// use payego_primitives::error::ApiError;
// use std::sync::Arc;
// use tracing::info;
// use payego_primitives::models::app_config::AppConfig;
// use payego_primitives::models::app_state::app_state::AppState;
//
//
// use diesel::prelude::*;
// use reqwest::Client;
// use serde_json::Value;
// use tracing::log::error;
// use payego_primitives::models::bank::{Bank, NewBank};
// use payego_primitives::models::enum_types::CurrencyCode;
// use payego_primitives::schema::banks;
//
//
// #[utoipa::path(
//     post,
//     path = "/api/bank/init",
//     responses(
//         (status = 201, description = "Banks initialized successfully",),
//         (status = 400, description = "Bank initialization failed"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Auth"
// )]
// pub async fn initialize_banks(
//     State(state): State<Arc<AppState>>,
// ) -> Result<StatusCode, ApiError>  {
//     let mut conn = state
//         .db
//         .get()
//         .map_err(|e| {
//             error!("Database connection failed: {}", e);
//             ApiError::DatabaseConnection(e.to_string())
//         })?;
//
//     // Check if banks table is populated
//     let bank_count: i64 = banks::table
//         .count()
//         .get_result(&mut conn)
//         .map_err(|e| {
//             error!("Failed to count banks: {}", e);
//             ApiError::Database(e)
//         })?;
//
//     // Assume at least 10 banks for a valid population (Paystack typically returns ~25 banks for Nigeria)
//     const MIN_BANKS: i64 = 10;
//     if bank_count >= MIN_BANKS {
//         info!("Banks table already populated with {} banks, skipping Paystack fetch", bank_count);
//         return Ok(StatusCode::OK);
//     }
//
//     // Fetch from Paystack
//     let paystack_key = std::env::var("PAYSTACK_SECRET_KEY").map_err(|_| {
//         error!("PAYSTACK_SECRET_KEY not set");
//         ApiError::Token("Paystack key not set".to_string())
//     })?;
//     let client = Client::new();
//     let url = "https://api.paystack.co/bank?country=nigeria";
//     let resp = client
//         .get(url)
//         .header("Authorization", format!("Bearer {}", paystack_key))
//         .send()
//         .await
//         .map_err(|e| {
//             error!("Paystack banks API error: {}", e);
//             ApiError::Payment(format!("Paystack banks API error: {}", e))
//         })?;
//
//     let status = resp.status();
//     let body = resp.json::<Value>().await.map_err(|e| {
//         error!("Paystack response parsing error: {}", e);
//         ApiError::Payment(format!("Paystack response error: {}", e))
//     })?;
//
//     if !status.is_success() || body["status"].as_bool().unwrap_or(false) == false {
//         let message = body["message"]
//             .as_str()
//             .unwrap_or("Unknown Paystack error")
//             .to_string();
//         error!("Paystack banks fetch failed: {}", message);
//         return Err(ApiError::Payment(format!("Paystack banks fetch failed: {}", message)).into());
//     }
//
//     let banks_data = body["data"].as_array().ok_or_else(|| {
//         error!("Invalid Paystack response: missing banks data");
//         ApiError::Payment("Invalid Paystack response".to_string())
//     })?;
//
//     let mut banks: Vec<NewBank> = Vec::new();
//     let mut skipped = 0;
//     for bank in banks_data.iter() {
//         let id = bank["id"].as_i64();
//         let name = bank["name"].as_str().map(|s| s.to_string());
//         let code = bank["code"].as_str().map(|s| s.to_string());
//         let currency = bank["currency"].as_str().map(|s| s.to_string());
//         let country = bank["country"].as_str().map(|s| s.to_string());
//         // let gateway = bank["gateway"].as_str().map(|s| s.to_string());
//         // let pay_with_bank = bank["pay_with_bank"].as_bool();
//         let is_active = bank["is_active"].as_bool();
//
//         match (id, name, code, currency, country, is_active) {
//             (Some(id), Some(name), Some(code), Some(currenc), Some(country), Some(is_active)) => {
//                 banks.push(NewBank {
//                     id,
//                     name,
//                     code,
//                     currency: CurrencyCode::parse(&*currenc)?,
//                     country,
//                     is_active,
//                 });
//             }
//             _ => {
//                 error!("Invalid bank data: {:?}", bank);
//                 skipped += 1;
//             }
//         }
//     }
//
//     if banks.is_empty() {
//         error!("No valid banks fetched from Paystack");
//         return Err(ApiError::Payment("No valid banks fetched from Paystack".to_string()).into());
//     }
//
//     // Insert banks into database with ON CONFLICT DO NOTHING
//     let inserted_count = diesel::insert_into(banks::table)
//         .values(&banks)
//         .on_conflict(banks::code)
//         .do_nothing()
//         .execute(&mut conn)
//         .map_err(|e| {
//             error!("Failed to insert banks into database: {}", e);
//             ApiError::Database(e)
//         })?;
//
//     info!(
//         "Inserted {} banks into database during startup, skipped {} ({} invalid, {} duplicates)",
//         inserted_count,
//         banks.len() - inserted_count + skipped,
//         skipped,
//         banks.len() - inserted_count
//     );
//     Ok(StatusCode::OK)
// }

//==================================

use axum::extract::State;
use diesel::prelude::*;
use http::StatusCode;
use payego_core::services::bank_service::BankService;
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;


use diesel::prelude::*;


#[utoipa::path(
    post,
    path = "/api/bank/init",
    responses(
        (status = 201, description = "Banks initialized"),
        (status = 200, description = "Banks already initialized"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn initialize_banks(
    // state: &Arc<AppState>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, ApiError> {

    let mut conn = state.db.get().map_err(|e| {
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let initialized = BankService::initialize_banks(&state, &mut conn).await?;

    Ok(if initialized {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    })
}



