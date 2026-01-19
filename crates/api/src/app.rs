use crate::config::swagger_config::ApiDoc;
use crate::handlers::{
    all_banks::all_banks, bank::add_bank_account, current_user::current_user_details,
    get_transaction::get_transactions, health::health_check, initialize_banks::initialize_banks,
    internal_conversion::convert_currency, login::login, logout::logout,
    paypal_capture::paypal_capture, paystack_webhook::paystack_webhook, register::register,
    resolve_account::resolve_account, stripe_webhook::stripe_webhook, top_up::top_up,
    transaction::get_user_transaction, transfer_external::transfer_external,
    transfer_internal::transfer_internal, user_bank_accounts::user_bank_accounts,
    user_wallets::get_wallets, withdraw::withdraw,
};
use axum::{middleware, response::IntoResponse, routing::post, Router};
// use payego_primitives::config::security_config::auth_middleware;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use payego_primitives::config::security_config::SecurityConfig;
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    // rate limiting configuration
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2) // 2 requests per second = 120 per minute
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    // public routes (no authentication)
    let public_router = create_public_routers();

    // protected routes (require JWT authentication)
    let protected_router = create_secured_routers(&state);

    let mut router = Router::new()
        .merge(public_router)
        .merge(protected_router)
        .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024)) // 2MB limit
        .layer(middleware::from_fn(https_redirect_middleware))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(TraceLayer::new_for_http()),
        );

    // disable rate limiting in test environment to avoid "Unable To Extract Key!" errors
    if std::env::var("APP_ENV").unwrap_or_default() != "test" {
        router = router.layer(GovernorLayer::new(governor_conf));
    }

    router.with_state(state)
}

fn create_secured_routers(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    let protected_router = Router::new()
        .route(
            "/api/current_user",
            axum::routing::get(current_user_details),
        )
        .route("/api/bank_accounts", axum::routing::get(user_bank_accounts))
        .route("/api/wallets", axum::routing::get(get_wallets))
        .route("/api/transactions", axum::routing::get(get_transactions))
        .route(
            "/api/transactions/{transaction_id}",
            axum::routing::get(get_user_transaction),
        )
        .route("/api/top_up", axum::routing::post(top_up))
        .route("/api/logout", axum::routing::post(logout))
        .route(
            "/api/convert_currency",
            axum::routing::post(convert_currency),
        )
        .route("/api/paypal/capture", axum::routing::post(paypal_capture))
        .route(
            "/api/transfer/internal",
            axum::routing::post(transfer_internal),
        )
        .route(
            "/api/transfer/external",
            axum::routing::post(transfer_external),
        )
        .route("/api/add_bank", axum::routing::post(add_bank_account))
        .route("/api/withdraw", axum::routing::post(withdraw))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            SecurityConfig::auth_middleware,
        ));
    protected_router
}

fn create_public_routers() -> Router<Arc<AppState>> {
    let public_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/register", post(register))
        .route("/api/login", axum::routing::post(login))
        .route(
            "/api/auth/refresh",
            axum::routing::post(crate::handlers::refresh_token::refresh_token),
        )
        .route("/api/webhook/stripe", post(stripe_webhook))
        .route("/webhooks/paystack", post(paystack_webhook))
        .route("/api/bank/init", axum::routing::post(initialize_banks))
        .route("/api/banks", axum::routing::get(all_banks))
        .route("/api/resolve_account", axum::routing::get(resolve_account))
        .route("/api/health", axum::routing::get(health_check));
    public_router
}

async fn https_redirect_middleware(
    req: axum::extract::Request,
    next: middleware::Next,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    // Check if we are in production
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    if env == "production" {
        let headers = req.headers();
        let proto = headers
            .get("x-forwarded-proto")
            .and_then(|h| h.to_str().ok());

        if let Some("http") = proto {
            let host = headers
                .get("host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("localhost");

            let uri = req.uri();
            let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("");
            let redirect_url = format!("https://{}{}", host, path_and_query);

            return Ok(axum::response::Redirect::permanent(&redirect_url).into_response());
        }
    }

    Ok(next.run(req).await)
}
