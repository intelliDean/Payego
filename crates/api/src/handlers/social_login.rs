// use payego_primitives::error::ApiError;
// use axum::response::IntoResponse;
// use axum::{Json, extract::State, http::StatusCode};
// use diesel::prelude::*;
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use diesel::result::Error::DatabaseError;
// use utoipa::ToSchema;
// use uuid::Uuid;
// use payego_primitives::config::security_config::create_token;
// use payego_primitives::models::{AppState, NewUser, NewWallet, RegisterResponse};
// use reqwest::Client;
// use serde_json::Value;
//
// #[derive(Deserialize, ToSchema)]
// pub struct SocialLoginRequest {
//     id_token: String,
//     provider: String,
// }
//
// #[utoipa::path(
//     post,
//     path = "/api/social_login",
//     request_body = SocialLoginRequest,
//     responses(
//         (status = 200, description = "Social login successful", body = RegisterResponse),
//         (status = 400, description = "Invalid token or provider"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Auth"
// )]
// pub async fn social_login(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<SocialLoginRequest>,
// ) -> Result<_, ApiError> {
//     if payload.provider != "google" {
//         return Err((StatusCode::BAD_REQUEST, "Unsupported provider".to_string()));
//     }
//
//     // Validate Google ID token
//     let client = Client::new();
//     let response: Value = client
//         .get("https://www.googleapis.com/oauth2/v3/tokeninfo")
//         .query(&[("id_token", &payload.id_token)])
//         .send()
//         .await
//         .map_err(|e| {
//             tracing::error!("Failed to validate Google token: {}", e);
//             (StatusCode::BAD_REQUEST, "Invalid token".to_string())
//         })?
//         .json()
//         .await
//         .map_err(|e| {
//             tracing::error!("Failed to parse Google token response: {}", e);
//             (StatusCode::BAD_REQUEST, "Invalid token response".to_string())
//         })?;
//
//     let email = response.get("email").and_then(|e| e.as_str()).ok_or_else(|| {
//         tracing::error!("No email in Google token response");
//         (StatusCode::BAD_REQUEST, "Invalid token".to_string())
//     })?;
//     let sub = response.get("sub").and_then(|s| s.as_str()).ok_or_else(|| {
//         tracing::error!("No sub in Google token response");
//         (StatusCode::BAD_REQUEST, "Invalid token".to_string())
//     })?;
//
//     let conn = &mut state.db.get().map_err(|e: r2d2::Error| {
//         tracing::error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     // Check if user exists or create new
//     let user_id: Uuid = conn
//         .transaction(|conn| {
//
//             let existing_user: Option<Uuid> = payego_primitives::schema::users::table
//                 .filter(payego_primitives::schema::users::email.eq(email))
//                 .select(payego_primitives::schema::users::id)
//                 .first(conn)
//                 .optional()
//                 .map_err(|e| {
//                     tracing::error!("Database error: {}", e);
//                     ApiError::from(e)
//                 })?;
//
//             if let Some(user_id) = existing_user {
//                 Ok(user_id)
//             } else {
//                 // Create new user
//                 let user_id: Uuid = diesel::insert_into(payego_primitives::schema::users::table)
//                     .values(NewUser {
//                         email: email.to_string(),
//                         password_hash: "".to_string(),
//                         username: None,
//                     })
//                     .returning(payego_primitives::schema::users::id)
//                     .get_result(conn)
//                     .map_err(|e| {
//                         tracing::error!("Database error: {}", e);
//                         ApiError::from(e)
//                     })?;
//
//                 // Create default USD wallet
//                 diesel::insert_into(payego_primitives::schema::wallets::table)
//                     .values(NewWallet {
//                         user_id,
//                         balance: 0,
//                         currency: "USD".to_string(),
//                     })
//                     .execute(conn)
//                     .map_err(|e| {
//                         tracing::error!("Database error: {}", e);
//                         ApiError::from(e)
//                     });
//
//                 Ok(user_id)
//             }
//         })
//         .map_err(ApiError::Database);
//
//     // Generate JWT token
//     let token = create_token(&state, &user_id.to_string())?;
//
//     tracing::info!("Social login successful: email={}", email);
//
//     Ok((
//         StatusCode::OK,
//         Json(RegisterResponse {
//             token,
//             user_email: email.to_string(),
//         }),
//     ))
// }
