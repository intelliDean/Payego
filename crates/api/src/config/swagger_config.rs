use crate::handlers::{
    add_bank::__path_add_bank_account, all_banks::__path_all_banks,
    current_user::__path_current_user_details, get_transaction::__path_get_transactions,
    health::__path_health_check, internal_conversion::__path_convert_currency, login::__path_login,
    logout::__path_logout, paypal_capture::__path_paypal_capture,
    paypal_order::__path_get_paypal_order, paystack_webhook::__path_paystack_webhook,
    refresh_token::__path_refresh_token, register::__path_register,
    resolve_account::__path_resolve_account, stripe_webhook::__path_stripe_webhook,
    top_up::__path_top_up, transfer_external::__path_transfer_external,
    transfer_internal::__path_transfer_internal, user_bank_accounts::__path_user_bank_accounts,
    user_transaction::__path_get_user_transaction, user_wallets::__path_get_user_wallets,
    withdraw::__path_withdraw, resolve_user::__path_resolve_user, delete_bank::__path_delete_bank_account
};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::SecurityRequirement;
use utoipa::{Modify, OpenApi};
use payego_primitives::error::ApiErrorResponse;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Payego API",
        version = env!("CARGO_PKG_VERSION"), //"1.0.0"
        description = "Public API for Payego payments, wallets, transfers, and integrations.",
        contact(
            name = "Payego Engineering",
            url = "https://dean8ix.vercel.app/",
            email = "o.michaeldean@gmail.com"
        ),
        license(
            name = "Proprietary - All Rights Reserved",
            url = "https://dean8ix.vercel.app/"
        )
    ),
    paths(
        register,
        login,
        logout,
        health_check,
        current_user_details,
        top_up,
        withdraw,
        transfer_internal,
        transfer_external,
        convert_currency,
        resolve_account,
        add_bank_account,
        user_bank_accounts,
        all_banks,
        get_transactions,
        get_user_transaction,
        get_paypal_order,
        paypal_capture,
        paystack_webhook,
        refresh_token,
        stripe_webhook,
        get_user_wallets,
        resolve_user,
        delete_bank_account
    ),
    tags(
        (name = "Authentication", description = "User registration, login, logout, current user info"),
        (name = "Wallet", description = "Wallet balances, top-ups, withdrawals, currency conversion"),
        (name = "Bank", description = "Linking, resolving, listing bank accounts"),
        (name = "Payments", description = "Payment gateways: PayPal, Paystack, Stripe"),
        (name = "Transactions", description = "Transaction history & details"),
        (name = "Webhooks", description = "Inbound webhook endpoints from payment providers"),
    ),
    components(
        schemas(ApiErrorResponse)
    ),
    modifiers(&SecurityAddon),
    security(
        ("bearerAuth" = []),
    ),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();

        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT Bearer token. Format: `Bearer <token>`"))
                    .build(),
            ),
        );

        openapi.security = Some(vec![SecurityRequirement::new::<&str, Vec<_>, String>(
            "bearerAuth",
            vec![],
        )]);
    }
}

