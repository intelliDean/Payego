// @generated automatically by Diesel CLI.

diesel::table! {
    bank_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 10]
        bank_code -> Varchar,
        #[max_length = 20]
        account_number -> Varchar,
        #[max_length = 255]
        account_name -> Nullable<Varchar>,
        #[max_length = 255]
        bank_name -> Nullable<Varchar>,
        #[max_length = 50]
        paystack_recipient_code -> Nullable<Varchar>,
        is_verified -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    banks (id) {
        id -> Int8,
        name -> Varchar,
        code -> Varchar,
        currency -> Varchar,
        country -> Varchar,
        gateway -> Nullable<Varchar>,
        pay_with_bank -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
    }
}

diesel::table! {
    blacklisted_tokens (token) {
        token -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    transactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        recipient_id -> Nullable<Uuid>,
        amount -> Int8,
        #[max_length = 50]
        transaction_type -> Varchar,
        #[max_length = 3]
        currency -> Varchar,
        #[max_length = 50]
        status -> Varchar,
        #[max_length = 50]
        provider -> Nullable<Varchar>,
        description -> Nullable<Text>,
        reference -> Uuid,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Text,
        #[max_length = 100]
        username -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    wallets (id) {
        id -> Uuid,
        user_id -> Uuid,
        balance -> Int8,
        #[max_length = 3]
        currency -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(bank_accounts -> users (user_id));
diesel::joinable!(wallets -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    bank_accounts,
    banks,
    blacklisted_tokens,
    transactions,
    users,
    wallets,
);
