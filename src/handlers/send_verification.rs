// use crate::error::ApiError;
// use crate::models::models::{AppState, RegisterResponse};
// use axum::response::IntoResponse;
// use axum::{extract::State, http::StatusCode, Json};
// use chrono::{Duration, Utc};
// use diesel::prelude::*;
// use lettre::{
//     message::header::ContentType, transport::smtp::authentication::Credentials, Message,
//     SmtpTransport, Transport,
// };
// use rand::distributions::{Alphanumeric};
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// // use axum::extract::path::ErrorKind::Message;
// // use headers::ContentType;
// use utoipa::ToSchema;
// use uuid::Uuid;
//
// #[derive(Deserialize, ToSchema)]
// pub struct SendVerificationRequest {
//     email: String,
// }
//
// #[derive(Deserialize, ToSchema)]
// pub struct VerifyEmailRequest {
//     email: String,
//     code: String,
// }
//
// #[derive(Serialize, ToSchema)]
// pub struct VerifyEmailResponse {
//     message: String,
// }
//
// #[utoipa::path(
//     post,
//     path = "/api/send_verification",
//     request_body = SendVerificationRequest,
//     responses(
//         (status = 200, description = "Verification code sent successfully"),
//         (status = 400, description = "User not found or already verified"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Auth"
// )]
// pub async fn send_verification(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<SendVerificationRequest>,
// ) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
//     let conn = &mut state.db.get().map_err(|e| {
//         tracing::error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     // Find user by email
//     let user: Option<(Uuid, bool)> = crate::schema::users::table
//         .filter(crate::schema::users::email.eq(&payload.email))
//         .select((crate::schema::users::id, crate::schema::users::is_verified))
//         .first(conn)
//         .optional()
//         .map_err(|e| {
//             tracing::error!("Database error: {}", e);
//             ApiError::Database(e)
//         })?;
//
//     if let Some((user_id, is_verified)) = user {
//         if is_verified {
//             return Err((
//                 StatusCode::BAD_REQUEST,
//                 "Email already verified".to_string(),
//             ));
//         }
//
//         // Generate 6-digit code
//         let code = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
//         let expires_at = Utc::now() + Duration::minutes(15);
//
//         // Store code in verification_codes table
//         diesel::insert_into(crate::schema::verification_codes::table)
//             .values((
//                 crate::schema::verification_codes::user_id.eq(user_id),
//                 crate::schema::verification_codes::code.eq(&code),
//                 crate::schema::verification_codes::expires_at.eq(expires_at),
//             ))
//             .execute(conn)
//             .map_err(|e| {
//                 tracing::error!("Database error: {}", e);
//                 ApiError::Database(e)
//             })?;
//
//         // Send email (configure SMTP details in AppState or environment)
//         let email = Message::builder()
//             .from("Payego <no-reply@payego.com>".parse().unwrap())
//             .to(payload.email.parse().unwrap())
//             .subject("Payego Email Verification")
//             .header(ContentType::TEXT_PLAIN)
//             .body(format!("Your verification code is: {}", code))
//             .unwrap();
//
//         let smtp_credentials = Credentials::new(
//             env!("SMTP_USERNAME").to_string(),
//             env!("SMTP_PASSWORD").to_string(),
//         );
//         let mailer = SmtpTransport::relay(env!("SMTP_HOST"))
//             .unwrap()
//             .credentials(smtp_credentials)
//             .build();
//
//         mailer.send(&email).map_err(|e| {
//             tracing::error!("Failed to send email: {}", e);
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 "Failed to send email".to_string(),
//             )
//         })?;
//
//         tracing::info!("Verification code sent to {}", payload.email);
//         Ok((
//             StatusCode::OK,
//             Json(serde_json::json!({"message": "Verification code sent"})),
//         ))
//     } else {
//         Err((StatusCode::BAD_REQUEST, "User not found".to_string()))
//     }
// }
//
// #[utoipa::path(
//     post,
//     path = "/api/verify_email",
//     request_body = VerifyEmailRequest,
//     responses(
//         (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
//         (status = 400, description = "Invalid or expired code"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Auth"
// )]
// pub async fn verify_email(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<VerifyEmailRequest>,
// ) -> Result<(StatusCode, Json<VerifyEmailResponse>), (StatusCode, String)> {
//     let conn = &mut state.db.get().map_err(|e| {
//         tracing::error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     // Find user
//     let user: Option<(Uuid, bool)> = crate::schema::users::table
//         .filter(crate::schema::users::email.eq(&payload.email))
//         .select((crate::schema::users::id, crate::schema::users::is_verified))
//         .first(conn)
//         .optional()
//         .map_err(|e| {
//             tracing::error!("Database error: {}", e);
//             ApiError::Database(e)
//         })?;
//
//     if let Some((user_id, is_verified)) = user {
//         if is_verified {
//             return Err((
//                 StatusCode::BAD_REQUEST,
//                 "Email already verified".to_string(),
//             ));
//         }
//
//         // Find verification code
//         let code: Option<(String, chrono::DateTime<Utc>)> =
//             crate::schema::verification_codes::table
//                 .filter(crate::schema::verification_codes::user_id.eq(user_id))
//                 .select((
//                     crate::schema::verification_codes::code,
//                     crate::schema::verification_codes::expires_at,
//                 ))
//                 .order_by(crate::schema::verification_codes::created_at.desc())
//                 .first(conn)
//                 .optional()
//                 .map_err(|e| {
//                     tracing::error!("Database error: {}", e);
//                     ApiError::Database(e)
//                 })?;
//
//         if let Some((stored_code, expires_at)) = code {
//             if Utc::now() > expires_at {
//                 return Err((
//                     StatusCode::BAD_REQUEST,
//                     "Verification code expired".to_string(),
//                 ));
//             }
//             if stored_code != payload.code {
//                 return Err((
//                     StatusCode::BAD_REQUEST,
//                     "Invalid verification code".to_string(),
//                 ));
//             }
//
//             // Mark user as verified
//             diesel::update(
//                 crate::schema::users::table.filter(crate::schema::users::id.eq(user_id)),
//             )
//             .set(crate::schema::users::is_verified.eq(true))
//             .execute(conn)
//             .map_err(|e| {
//                 tracing::error!("Database error: {}", e);
//                 ApiError::Database(e)
//             })?;
//
//             // Delete used code
//             diesel::delete(
//                 crate::schema::verification_codes::table
//                     .filter(crate::schema::verification_codes::user_id.eq(user_id)),
//             )
//             .execute(conn)
//             .map_err(|e| {
//                 tracing::error!("Database error: {}", e);
//                 ApiError::Database(e)
//             })?;
//
//             tracing::info!("Email verified for {}", payload.email);
//             Ok((
//                 StatusCode::OK,
//                 Json(VerifyEmailResponse {
//                     message: "Email verified successfully".to_string(),
//                 }),
//             ))
//         } else {
//             Err((
//                 StatusCode::BAD_REQUEST,
//                 "No verification code found".to_string(),
//             ))
//         }
//     } else {
//         Err((StatusCode::BAD_REQUEST, "User not found".to_string()))
//     }
// }
