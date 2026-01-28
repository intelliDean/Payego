DROP INDEX IF EXISTS idx_audit_logs_user_id_created_at;
DROP TABLE IF EXISTS verification_tokens;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified_at;
