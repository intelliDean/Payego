// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "currency_code"))]
    pub struct CurrencyCode;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_provider"))]
    pub struct PaymentProvider;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_state"))]
    pub struct PaymentState;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_intent"))]
    pub struct TransactionIntent;
}

diesel::table! {
    audit_logs (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        event_type -> Text,
        target_type -> Nullable<Text>,
        target_id -> Nullable<Text>,
        metadata -> Jsonb,
        ip_address -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    bank_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        bank_code -> Text,
        account_number -> Text,
        account_name -> Nullable<Text>,
        bank_name -> Nullable<Text>,
        provider_recipient_id -> Nullable<Text>,
        is_verified -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CurrencyCode;

    banks (id) {
        id -> Int8,
        name -> Text,
        code -> Text,
        currency -> CurrencyCode,
        country -> Varchar,
        is_active -> Bool,
    }
}

diesel::table! {
    blacklisted_tokens (jti) {
        jti -> Text,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        token_hash -> Text,
        expires_at -> Timestamptz,
        revoked -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionIntent;
    use super::sql_types::CurrencyCode;
    use super::sql_types::PaymentState;
    use super::sql_types::PaymentProvider;

    transactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        counterparty_id -> Nullable<Uuid>,
        intent -> TransactionIntent,
        amount -> Int8,
        currency -> CurrencyCode,
        txn_state -> PaymentState,
        provider -> Nullable<PaymentProvider>,
        provider_reference -> Nullable<Text>,
        idempotency_key -> Text,
        reference -> Uuid,
        description -> Nullable<Text>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        password_hash -> Text,
        username -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        email_verified_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    verification_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        token_hash -> Text,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    wallet_ledger (id) {
        id -> Uuid,
        wallet_id -> Uuid,
        transaction_id -> Uuid,
        amount -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CurrencyCode;

    wallets (id) {
        id -> Uuid,
        user_id -> Uuid,
        currency -> CurrencyCode,
        balance -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(audit_logs -> users (user_id));
diesel::joinable!(bank_accounts -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(verification_tokens -> users (user_id));
diesel::joinable!(wallet_ledger -> transactions (transaction_id));
diesel::joinable!(wallet_ledger -> wallets (wallet_id));
diesel::joinable!(wallets -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    audit_logs,
    bank_accounts,
    banks,
    blacklisted_tokens,
    refresh_tokens,
    transactions,
    users,
    verification_tokens,
    wallet_ledger,
    wallets,
);
