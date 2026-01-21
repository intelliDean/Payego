use crate::config::swagger_config::ApiDoc;
use crate::handlers::{
    add_bank::add_bank_account,
    all_banks::all_banks,
    current_user::current_user_details,
    get_transaction::get_transactions,
    health::health_check,
    // initialize_banks::initialize_banks,
    internal_conversion::convert_currency,
    login::login,
    logout::logout,
    paypal_capture::paypal_capture,
    paystack_webhook::paystack_webhook,
    register::register,
    resolve_account::resolve_account,
    stripe_webhook::stripe_webhook,
    top_up::top_up,
    transfer_external::transfer_external,
    transfer_internal::transfer_internal,
    user_bank_accounts::user_bank_accounts,
    user_transaction::get_user_transaction,
    user_wallets::get_user_wallets,
    withdraw::withdraw,
};
use axum::{middleware, response::IntoResponse, routing::post, Router};
// use payego_primitives::config::security_config::auth_middleware;
use payego_primitives::models::app_state::app_state::AppState;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::paypal_order::get_paypal_order;
use crate::handlers::refresh_token::refresh_token;
use payego_primitives::config::security_config::SecurityConfig;
use tower::{ServiceBuilder, ServiceExt};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_governor::governor::GovernorConfig;
use tower_http::{
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};

const REQUESTS_PER_SECOND: u64 = 2;
const BURST_SIZE: u32 = 10;

//this fails at compile time and not runtime
const _: () = assert!(REQUESTS_PER_SECOND > 0);
const _: () = assert!(BURST_SIZE > 0);


pub fn create_router(state: Arc<AppState>) -> Router {
    // rate limiting configuration

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(REQUESTS_PER_SECOND) // 2 requests per second = 120 per minute
            .burst_size(BURST_SIZE)
            .finish()
            .expect("Invalid rate limiter configuration: "),
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
            "/api/user/current",
            axum::routing::get(current_user_details),
        )
        .route("/api/user/banks", axum::routing::get(user_bank_accounts))
        .route("/api/user/wallets", axum::routing::get(get_user_wallets))
        .route(
            "/api/user/transactions",
            axum::routing::get(get_transactions),
        )
        .route(
            "/api/paypal/order/{order_id}",
            axum::routing::get(get_paypal_order),
        )
        .route(
            "/api/transactions/{transaction_id}",
            axum::routing::get(get_user_transaction),
        )
        .route("/api/wallet/top_up", axum::routing::post(top_up))
        .route("/api/auth/logout", axum::routing::post(logout))
        .route(
            "/api/wallets/convert",
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
        .route("/api/banks/add", axum::routing::post(add_bank_account))
        .route(
            "/api/wallet/withdraw/{bank_account_id}",
            axum::routing::post(withdraw),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            SecurityConfig::auth_middleware,
        ));
    protected_router
}

fn create_public_routers() -> Router<Arc<AppState>> {
    let public_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", axum::routing::post(login))
        .route("/api/auth/refresh", post(refresh_token))
        .route("/api/webhooks/stripe", post(stripe_webhook))
        .route("/api/webhooks/paystack", post(paystack_webhook))
        // .route("/api/bank/init", axum::routing::post(initialize_banks))
        .route("/api/banks/all", axum::routing::get(all_banks))
        .route("/api/bank/resolve", axum::routing::get(resolve_account))
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
