-- This file should undo anything in `up.sql`
-- Drop indexes
DROP INDEX IF EXISTS idx_bank_accounts_user_id;
DROP INDEX IF EXISTS idx_transactions_reference;
DROP INDEX IF EXISTS idx_transactions_recipient_id;
DROP INDEX IF EXISTS idx_transactions_user_id;

-- Drop tables in reverse order to respect foreign key constraints
DROP TABLE IF EXISTS bank_accounts;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS wallets;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS banks;
DROP TABLE refresh_tokens;