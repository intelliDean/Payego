CREATE TYPE currency_code AS ENUM (
    'usd','ngn','gbp','eur','cad','aud','jpy','chf','cny',
    'sek','nzd','mxn','sgd','hkd','nok','krw','try',
    'inr','brl','zar'
    );


CREATE TYPE transaction_intent AS ENUM (
    'top_up',
    'payout',
    'transfer',
    'conversion'
    );

CREATE TYPE payment_state AS ENUM (
    'pending',
    'requires_action',
    'completed',
    'failed',
    'cancelled'
    );

CREATE TYPE payment_provider AS ENUM (
    'stripe',
    'paypal',
    'paystack',
    'internal'
    );

CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

                       email TEXT NOT NULL UNIQUE,
                       password_hash TEXT NOT NULL,
                       username TEXT UNIQUE,

                       created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE wallets (
                         id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

                         user_id UUID NOT NULL
                             REFERENCES users (id) ON DELETE CASCADE,

                         currency currency_code NOT NULL,

    -- Cached balance (ledger is source of truth)
                         balance BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),

                         created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                         updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                         UNIQUE (user_id, currency)
);

CREATE TABLE transactions (
                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Actor
                              user_id UUID NOT NULL
                                  REFERENCES users (id) ON DELETE CASCADE,

    -- Receiver (nullable depending on intent)
                              counterparty_id UUID
                                  REFERENCES users (id),

                              intent transaction_intent NOT NULL,

    -- Always positive, direction handled by ledger
                              amount BIGINT NOT NULL CHECK (amount > 0),
                              currency currency_code NOT NULL,

                              txn_state payment_state NOT NULL,

                              provider payment_provider,
                              provider_reference TEXT UNIQUE,

    -- Safety & deduplication
                              idempotency_key TEXT NOT NULL,
                              reference UUID NOT NULL UNIQUE,

                              description TEXT,
                              metadata JSONB NOT NULL DEFAULT '{}' CHECK (jsonb_typeof(metadata) = 'object'),

                              created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                              updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                              UNIQUE (user_id, idempotency_key)
);

CREATE TABLE wallet_ledger (
                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

                               wallet_id UUID NOT NULL
                                   REFERENCES wallets (id) ON DELETE CASCADE,

                               transaction_id UUID NOT NULL
                                   REFERENCES transactions (id) ON DELETE CASCADE,

    -- Signed amount (+ credit, - debit)
                               amount BIGINT NOT NULL,

                               created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                               UNIQUE (wallet_id, transaction_id)
);

CREATE TABLE banks (
                       id BIGINT PRIMARY KEY,
                       name TEXT NOT NULL,
                       code TEXT NOT NULL UNIQUE,
                       currency currency_code NOT NULL,
                       country VARCHAR NOT NULL,
                       is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE bank_accounts (
                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

                               user_id UUID NOT NULL
                                   REFERENCES users (id) ON DELETE CASCADE,

                               bank_code TEXT NOT NULL
                                   REFERENCES banks (code),

                               account_number TEXT NOT NULL,
                               account_name TEXT,
                               bank_name TEXT,

                               provider_recipient_id TEXT,

                               is_verified BOOLEAN NOT NULL DEFAULT false,

                               created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                               updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                               UNIQUE (bank_code, account_number)
);

CREATE TABLE blacklisted_tokens(
                                   jti TEXT PRIMARY KEY,
                                   expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE refresh_tokens (
                                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

                                user_id UUID NOT NULL
                                    REFERENCES users (id) ON DELETE CASCADE,

                                token_hash TEXT NOT NULL,
                                expires_at TIMESTAMPTZ NOT NULL,

                                revoked BOOLEAN NOT NULL DEFAULT false,

                                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


CREATE INDEX idx_wallets_user ON wallets (user_id);

CREATE INDEX idx_transactions_user ON transactions (user_id);
CREATE INDEX idx_transactions_state ON transactions (txn_state);
CREATE INDEX idx_transactions_provider_ref ON transactions (provider_reference);

CREATE INDEX idx_ledger_wallet ON wallet_ledger (wallet_id);

CREATE INDEX idx_bank_accounts_user ON bank_accounts (user_id);

CREATE INDEX idx_blacklisted_tokens_expires ON blacklisted_tokens (expires_at);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens (user_id);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens (token_hash);
CREATE INDEX idx_refresh_tokens_expires ON refresh_tokens (expires_at);
CREATE INDEX idx_refresh_tokens_revoked ON refresh_tokens (revoked);

