use payego_primitives::error::ApiErrorResponse;
use axum::{
    extract::{Extension, State},
    Json,
};
use payego_core::services::wallet_service::{
    ApiError, AppState, Claims, WalletService, WalletsResponse,
};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/user/wallets",
    tag = "Wallet",
    summary = "List all user wallets and balances",
    description = "Retrieves the list of wallets belonging to the authenticated user, \
                   including balance, currency, wallet type (main, savings, bonus, etc.), \
                   status (active/frozen), last transaction timestamp, and other metadata. \
                   Typically used to display the user's total available funds, \
                   per-currency balances, or to let the user select a source wallet for transfers/payments. \
                   Only returns wallets associated with the current authenticated user.",
    operation_id = "getUserWallets",
    responses(
        ( status = 200, description = "Successfully retrieved list of user wallets with current balances", body = WalletsResponse),
        ( status = 401, description = "Unauthorized – missing, invalid, or expired authentication token", body = ApiErrorResponse),
        ( status = 403, description = "Forbidden – insufficient permissions (rare, usually role-based)", body = ApiErrorResponse),
        ( status = 429, description = "Too many requests – rate limit exceeded for wallet balance queries", body = ApiErrorResponse),
        ( status = 500, description = "Internal server error – failed to retrieve wallet information", body = ApiErrorResponse),
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_user_wallets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<WalletsResponse>, ApiError> {
    let wallets = WalletService::get_user_wallets(&state, &claims).await?;
    Ok(Json(wallets))
}

//
//
// pub async fn get_wallets(
//     State(state): State<Arc<AppState>>,
//     Extension(claims): Extension<Claims>,
// ) -> Result<Json<WalletsResponse>, ApiError> {
//     let user_id = Uuid::parse_str(&claims.sub)
//         .map_err(|_| ApiError::Auth(AuthError::InvalidToken("Invalid user ID".into())))?;
//
//     let mut conn = state.db.get().map_err(|e| {
//         error!("DB connection error: {}", e);
//         ApiError::DatabaseConnection(e.to_string())
//     })?;
//
//     let wallets = wallets::table
//         .filter(wallets::user_id.eq(user_id))
//         .order(wallets::created_at.asc())
//         .load::<Wallet>(&mut conn)
//         .map_err(ApiError::from)?;
//
//     let response = wallets.into_iter().map(WalletDto::from).collect::<Vec<_>>();
//
//     info!(
//         user_id = %user_id,
//         wallet_count = response.len(),
//         "Fetched user wallets"
//     );
//
//     Ok(Json(WalletsResponse { wallets: response }))
// }
