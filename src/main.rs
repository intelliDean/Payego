mod models;
mod schema;
mod config;
mod handlers;
mod utility;
mod error;

use axum::{middleware, response::IntoResponse, Router};
// use configs::security_config::auth_middleware;
use diesel::r2d2::Pool;
use diesel::{
    prelude::*,
    r2d2::ConnectionManager,
    PgConnection,
};
use tower_http::cors::{CorsLayer, Any};
use headers::HeaderMapExt;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::routing::{get, post};
use http::HeaderValue;
use tokio::net::TcpListener;
use tracing::log::info;
use utoipa::{OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use crate::config::security_config::{auth_middleware, JWTSecret};
use crate::config::swagger_config::ApiDoc;
use crate::handlers::current_user::get_current_user;
use crate::handlers::login::login;
use crate::handlers::paypal_capture::paypal_capture;
use crate::handlers::register::register;
use crate::handlers::stripe_webhook::stripe_webhook;
use crate::handlers::top_up::top_up;
use crate::models::user_models::AppState;
use tracing_subscriber;

// Database setup

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {

    // Initialize tracing with DEBUG level, outputting to terminal
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Include DEBUG and above
        .with_ansi(true) // Colorful output
        .init();

    info!("Starting application");

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let db_url = "postgres://postgres:%40Tiptop2059!@localhost:5432/todo";

    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .map_err(|e| eyre::eyre!("Failed to create pool: {}", e))?;


    let state = Arc::new(AppState {
        db: pool,
        jwt_secret: JWTSecret::new().jwt_secret
    });

    // Router setup
    // Public routes (no auth middleware)
    let public_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/register", post(register))
        .route("/api/webhook/stripe", post(stripe_webhook))
        .route("/api/login", post(login));

    // Protected routes (with auth middleware)
    let protected_router = Router::new()
        .route("/current_user", get(get_current_user))
        .route("/api/top_up", post(top_up))
        .route("/api/paypal/capture", post(paypal_capture))
    //     .route("/create_todo", post(create_todo))
    //     .route("/todos/get", post(get_todo))
    //     .route("/delete/{title}", post(delete_todo))
    //     .route("/todos/{title}", put(update_todo))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // CorsLayer::new()
    //     .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
    //     .allow_methods(Any)
    //     .allow_headers(Any)

    let app = Router::new()
        .merge(public_router)
        .merge(protected_router)
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>()?)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(state);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    info!("Server running on {:?}", addr);
    info!("Swagger UI available at {:?}/swagger-ui/index.html#/", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

