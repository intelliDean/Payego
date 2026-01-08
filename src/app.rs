use axum::{
    middleware,
    Router,
};
use std::sync::Arc;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;

use crate::config::security_config::auth_middleware;
use crate::config::swagger_config::ApiDoc;
use crate::handlers::internal_conversion::convert_currency;
use crate::handlers::resolve_account::resolve_account;
use crate::handlers::user_bank_accounts::user_bank_accounts;
use crate::handlers::user_wallets::get_wallets;
use crate::handlers::{
    all_banks::all_banks, bank::add_bank_account, current_user::current_user_details,
    get_transaction::get_transactions, login::login, paypal_capture::paypal_capture,
    paystack_webhook::paystack_webhook, register::register,
    stripe_webhook::stripe_webhook, top_up::top_up, transfer_external::external_transfer,
    transfer_internal::internal_transfer, withdraw::withdraw,
    initialize_banks::initialize_banks,
};
use crate::handlers::logout::logout;
use crate::handlers::transaction::get_user_transaction;
use crate::models::models::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    // Public routes (no authentication)
    let public_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/register", axum::routing::post(register))
        .route("/api/login", axum::routing::post(login))
        .route("/api/webhook/stripe", axum::routing::post(stripe_webhook))
        .route("/webhooks/paystack", axum::routing::post(paystack_webhook))
        .route("/api/bank/init", axum::routing::post(initialize_banks))
        .route("/api/banks", axum::routing::get(all_banks))
        .route("/api/resolve_account", axum::routing::get(resolve_account));

    // Protected routes (require JWT authentication)
    let protected_router = Router::new()
        .route(
            "/api/current_user",
            axum::routing::get(current_user_details),
        )
        .route("/api/bank_accounts", axum::routing::get(user_bank_accounts))
        .route("/api/wallets", axum::routing::get(get_wallets))
        .route("/api/transactions", axum::routing::get(get_transactions))
        .route("/api/transactions/{transaction_id}", axum::routing::get(get_user_transaction))
        .route("/api/top_up", axum::routing::post(top_up))
        .route("/api/logout", axum::routing::post(logout))
        .route(
            "/api/convert_currency",
            axum::routing::post(convert_currency),
        )
        .route("/api/paypal/capture", axum::routing::post(paypal_capture))
        .route(
            "/api/transfer/internal",
            axum::routing::post(internal_transfer),
        )
        .route(
            "/api/transfer/external",
            axum::routing::post(external_transfer),
        )
        .route("/api/add_bank", axum::routing::post(add_bank_account))
        .route("/api/withdraw", axum::routing::post(withdraw))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_router)
        .merge(protected_router)
        .with_state(state)
}
