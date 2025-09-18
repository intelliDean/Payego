use crate::handlers::{
    register::__path_register,
    login::__path_login,
    current_user::__path_get_current_user,
    top_up::__path_top_up,
    paypal_capture::__path_paypal_capture,
    stripe_webhook::__path_stripe_webhook,
};
use crate::models::user_models::*;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        register, login, get_current_user, top_up, paypal_capture, stripe_webhook
    ),
    components(schemas(RegisterRequest)),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Todos", description = "Todo management endpoints"),
        (name = "Users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Define the security scheme in components.securitySchemes
        if let Some(components) = openapi.components.as_mut() {
            components.security_schemes.insert(
                "bearerAuth".to_string(),
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
