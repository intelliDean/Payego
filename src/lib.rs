// Library entry point for Payego
// This exposes modules for testing while keeping main.rs as the binary entry point

pub mod app;
pub mod config;
pub mod error;
pub mod handlers;
pub mod logging;
pub mod models;
pub mod schema;
pub mod utility;

pub use models::AppState;
pub use error::ApiError;
