use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use payego_primitives::error::ApiError;
use payego_primitives::models::app_state::app_state::AppState;
use payego_primitives::models::dtos::register_dto::{RegisterRequest, RegisterResponse};
use std::sync::Arc;
use validator::Validate;
use payego_core::services::auth_service::register::RegisterService;

#[utoipa::path(
    post,
    path = "/api/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
    payload.validate()?;

    let response = RegisterService::register(&state, payload).await?;

    Ok((StatusCode::CREATED, Json(response)))
}






//
//
// #[utoipa::path(
//     post,
//     path = "/api/register",
//     request_body = RegisterRequest,
//     responses(
//         (status = 201, description = "User registered successfully", body = RegisterResponse),
//         (status = 400, description = "Invalid input"),
//         (status = 409, description = "Email already exists"),
//         (status = 500, description = "Internal server error")
//     ),
//     tag = "Authentication"
// )]
// pub async fn register(
//     State(state): State<Arc<AppState>>,
//     Json(payload): Json<RegisterRequest>,
// ) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
//     payload.validate().map_err(|e| {
//         error!("Validation error: {}", e);
//         ApiError::Validation(e)
//     })?;
//
//     let password = SecretString::new(payload.password.into());
//
//     let mut conn = state.db.get().map_err(|e| {
//         error!("Database connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     let password_hash = argon2id_hash_password(password)?;
//
//     //create the user
//     let new_user = NewUser {
//         email: &payload.email.clone(),
//         password_hash: &password_hash,
//         username: Option::from(payload.username.as_ref().unwrap().as_str()),
//     };
//
//     let user = diesel::insert_into(payego_primitives::schema::users::table)
//         .values(&new_user)
//         .get_result::<User>(&mut conn)
//         .map_err(|e| {
//             error!("User registration error: {}", e);
//             ApiError::from(e)
//         })?;
//
//     let token = create_token(&state, &user.id.to_string()).map_err(|e| {
//         error!("Token generation error: {}", e);
//         ApiError::Internal("Failed to generate token".to_string())
//     })?;
//
//     let refresh_token = AuthService::generate_refresh_token(&mut conn, user.id).map_err(|e| {
//         error!("Refresh token generation error: {}", e);
//         ApiError::from(e)
//     })?;
//
//     Ok((
//         StatusCode::CREATED,
//         Json(RegisterResponse {
//             token,
//             refresh_token,
//             user_email: user.email,
//         }),
//     ))
// }
//
// fn argon2id_hash_password(password: SecretBox<str>) -> Result<String, ApiError> {
//     //hash the password
//     let salt = SaltString::generate(&mut OsRng);
//     let argon2 = AuthService::create_argon2()?;
//     let password_hash = argon2
//         .hash_password(password.expose_secret().as_bytes(), &salt)
//         .map_err(|e| {
//             error!("Argon2 hashing error: {}", e);
//             ApiError::Internal("Encryption error".to_string())
//         })?
//         .to_string();
//     Ok(password_hash)
// }
