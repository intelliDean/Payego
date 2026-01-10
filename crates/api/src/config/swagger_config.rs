use crate::handlers::{
    all_banks::__path_all_banks, bank::__path_add_bank_account,
    current_user::__path_current_user_details, get_transaction::__path_get_transactions,
    initialize_banks::__path_initialize_banks, internal_conversion::__path_convert_currency,
    login::__path_login, logout::__path_logout, paypal_capture::__path_paypal_capture,
    paypal_order::__path_get_paypal_order, paystack_webhook::__path_paystack_webhook,
    register::__path_register, resolve_account::__path_resolve_account,
    stripe_webhook::__path_stripe_webhook, top_up::__path_top_up,
    transaction::__path_get_user_transaction, transfer_external::__path_transfer_external,
    transfer_internal::__path_transfer_internal, user_bank_accounts::__path_user_bank_accounts,
    user_wallets::__path_get_wallets, withdraw::__path_withdraw,
};
use payego_primitives::models::*;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        register, login, current_user_details, top_up, paypal_capture,
        stripe_webhook, transfer_internal, transfer_external,
        add_bank_account, withdraw, all_banks, paystack_webhook,
        initialize_banks, get_paypal_order, get_wallets,
        user_bank_accounts, convert_currency, resolve_account,
        get_transactions, get_user_transaction, logout
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
