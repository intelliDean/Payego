-- Users table
CREATE TABLE users
(
    id            UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    email         VARCHAR(255) UNIQUE      NOT NULL,
    password_hash TEXT                     NOT NULL,
    username      VARCHAR(100) UNIQUE,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Wallets table
CREATE TABLE wallets
(
    id         UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    user_id    UUID UNIQUE              NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    balance    BIGINT                   NOT NULL DEFAULT 0 CHECK (balance >= 0),
    currency   VARCHAR(3)               NOT NULL DEFAULT 'USD' CHECK (currency IN ('USD', 'NGN')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Transactions table
CREATE TABLE transactions
(
    id               UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    user_id          UUID                     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    recipient_id     UUID                     REFERENCES users (id) ON DELETE SET NULL,
    amount           BIGINT                   NOT NULL,
    transaction_type VARCHAR(50)              NOT NULL CHECK (transaction_type IN
                                                              ('topup_stripe', 'topup_paypal', 'internal_transfer_send',
                                                               'internal_transfer_receive', 'paystack_payout')),
    status           VARCHAR(50)              NOT NULL CHECK (status IN ('pending', 'completed', 'failed')),
    provider         VARCHAR(50) CHECK (provider IN ('stripe', 'paypal', 'paystack', NULL)),
    description      TEXT,
    reference        UUID UNIQUE NOT NULL,
    metadata         JSONB CHECK (jsonb_typeof(metadata) = 'object'),
    created_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Bank Accounts table
CREATE TABLE bank_accounts
(
    id                      UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    user_id                 UUID                     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    bank_code               VARCHAR(10)              NOT NULL,
    account_number          VARCHAR(20)              NOT NULL,
    account_name            VARCHAR(255),
    paystack_recipient_code VARCHAR(50),
    is_verified             BOOLEAN                  NOT NULL DEFAULT FALSE,
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_transactions_user_id ON transactions (user_id);
CREATE INDEX idx_transactions_recipient_id ON transactions (recipient_id);
CREATE INDEX idx_transactions_reference ON transactions (reference);
CREATE INDEX idx_bank_accounts_user_id ON bank_accounts (user_id);

-- Unique constraint on reference (when not null)
ALTER TABLE transactions
    ADD CONSTRAINT unique_reference UNIQUE (reference);