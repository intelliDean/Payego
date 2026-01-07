use crate::error::ApiError;
use crate::models::models::{AppState, NewUser, NewWallet, RegisterRequest, RegisterResponse};
use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use bcrypt::hash;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use crate::config::security_config::create_token;
use crate::error::ApiError::Bcrypt;

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Email or username already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, String)> {
    // Validate the payload first
    payload.validate().map_err(|e| {
        tracing::error!("Validation error: {}", e);
        ApiError::Validation(e)
    })?;

    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;

    let hashed = hash(&payload.password, 12).map_err(Bcrypt)?;

    // Clone the values we need before moving payload
    let email = payload.email.clone();
    let username = payload.username.clone();

    let user_id = conn
        .transaction(|conn| {
            // Check if email or username already exists first to avoid constraint errors
            let exists: bool = crate::schema::users::table
                .filter(
                    crate::schema::users::email
                        .eq(&email)
                        .or(crate::schema::users::username.eq(&username.as_deref().unwrap_or(""))),
                )
                .select(diesel::dsl::count_star())
                .first::<i64>(conn)
                .map(|count| count > 0)
                .map_err(|e| {
                    tracing::error!("Failed to check existing user: {}", e);
                    e  // Return DieselError to trigger transaction rollback
                })?;

            if exists {
                return Err(DieselError::RollbackTransaction);
            }

            if let Some(ref username) = username {
                let username_exists: bool = crate::schema::users::table
                    .filter(crate::schema::users::username.eq(username))
                    .select(diesel::dsl::count_star())
                    .first::<i64>(conn)
                    .map(|count| count > 0)
                    .map_err(ApiError::Database)
                    .unwrap_or(false);

                if username_exists {
                    return Err(DieselError::RollbackTransaction);
                }
            }

            // Insert user and return the generated ID
            let usr_id: Uuid = diesel::insert_into(crate::schema::users::table)
                .values(NewUser {
                    email: payload.email,
                    password_hash: hashed,
                    username: payload.username,
                })
                .returning(crate::schema::users::id)
                .get_result(conn)?;

            // Use the returned user_id for wallet creation
            diesel::insert_into(crate::schema::wallets::table)
                .values(NewWallet {
                    user_id: usr_id,
                    balance: 0,
                    currency: "USD".to_string(),
                })
                .execute(conn)?;

            Ok::<Uuid, DieselError>(usr_id)
        })
        .map_err(|e| match e {
            DieselError::RollbackTransaction => {
                let email_exists: bool = crate::schema::users::table
                    .filter(crate::schema::users::email.eq(&email))
                    .select(diesel::dsl::count_star())
                    .first::<i64>(conn)
                    .map(|count| count > 0)
                    .unwrap_or(false);

                if email_exists {
                    (StatusCode::BAD_REQUEST, "Email already exists".to_string())
                } else {
                    (StatusCode::BAD_REQUEST, "Username already exists".to_string())
                }
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
        })?;

    // Generate JWT token with proper error handling
    let token = create_token(&state, &user_id.to_string())?;

    tracing::info!("User registered: email={}", email);

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            token,
            user_email: email
        }),
    ))
}

//============



