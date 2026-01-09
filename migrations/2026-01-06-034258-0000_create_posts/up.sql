-- Your SQL goes here
-- Create users table
CREATE TABLE users
(
    id            UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    email         VARCHAR(255) UNIQUE      NOT NULL,
    password_hash TEXT                     NOT NULL,
    username      VARCHAR(100) UNIQUE,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create wallets table
CREATE TABLE wallets
(
    id         UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    user_id    UUID                     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    balance    BIGINT                   NOT NULL DEFAULT 0 CHECK (balance >= 0),
    currency   VARCHAR(3)               NOT NULL CHECK (currency IN
                                                        ('USD', 'NGN', 'GBP', 'EUR', 'CAD', 'AUD', 'JPY', 'CHF', 'CNY',
                                                         'SEK', 'NZD', 'MXN', 'SGD', 'HKD', 'NOK', 'KRW', 'TRY', 'INR',
                                                         'BRL', 'ZAR')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT wallets_user_currency_key UNIQUE (user_id, currency)
);

-- Create transactions table
CREATE TABLE transactions
(
    id               UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    user_id          UUID                     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    recipient_id     UUID                     REFERENCES users (id) ON DELETE SET NULL,
    amount           BIGINT                   NOT NULL,
    transaction_type VARCHAR(50)              NOT NULL CHECK (transaction_type IN
                                                              ('topup_stripe', 'topup_paypal', 'internal_transfer_send',
                                                               'internal_transfer_receive', 'paystack_payout',
                                                               'currency_conversion')),
    currency         VARCHAR(3)               NOT NULL CHECK (currency IN
                                                              ('USD', 'NGN', 'GBP', 'EUR', 'CAD', 'AUD', 'JPY', 'CHF',
                                                               'CNY',
                                                               'SEK', 'NZD', 'MXN', 'SGD', 'HKD', 'NOK', 'KRW', 'TRY',
                                                               'INR',
                                                               'BRL', 'ZAR')),
    status           VARCHAR(50)              NOT NULL CHECK (status IN ('pending', 'completed', 'failed')),
    provider         VARCHAR(50) CHECK (provider IN ('stripe', 'paypal', 'paystack', 'internal', NULL)),
    description      TEXT,
    reference        UUID UNIQUE              NOT NULL,
    metadata         JSONB CHECK (jsonb_typeof(metadata) = 'object'),
    created_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create bank_accounts table
CREATE TABLE bank_accounts
(
    id                      UUID PRIMARY KEY                  DEFAULT gen_random_uuid(),
    user_id                 UUID                     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    bank_code               VARCHAR(10)              NOT NULL,
    account_number          VARCHAR(20)              NOT NULL,
    account_name            VARCHAR(255),
    bank_name               VARCHAR(255),
    paystack_recipient_code VARCHAR(50),
    is_verified             BOOLEAN                  NOT NULL DEFAULT FALSE,
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE banks
(
    id            BIGINT PRIMARY KEY,
    name          VARCHAR NOT NULL,
    code          VARCHAR NOT NULL UNIQUE,
    currency      VARCHAR NOT NULL,
    country       VARCHAR NOT NULL,
    gateway       VARCHAR,
    pay_with_bank BOOLEAN,
    is_active     BOOLEAN
);

CREATE TABLE blacklisted_tokens
(
    token      VARCHAR     NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (token)
);



-- Create indexes
CREATE INDEX idx_transactions_user_id ON transactions (user_id);
CREATE INDEX idx_transactions_recipient_id ON transactions (recipient_id);
CREATE INDEX idx_transactions_reference ON transactions (reference);
CREATE INDEX idx_bank_accounts_user_id ON bank_accounts (user_id);
CREATE INDEX idx_blacklisted_tokens_expires_at ON blacklisted_tokens (expires_at);

-- Create function to update updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE
    ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_wallets_updated_at
    BEFORE UPDATE
    ON wallets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_transactions_updated_at
    BEFORE UPDATE
    ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_bank_accounts_updated_at
    BEFORE UPDATE
    ON bank_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
