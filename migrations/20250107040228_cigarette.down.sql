DROP VIEW IF EXISTS daily_smoking_summary;

DROP TRIGGER IF EXISTS update_smoking_logs_updated_at ON smoking_logs;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP FUNCTION IF EXISTS update_updated_at_column();

DROP INDEX IF EXISTS idx_smoking_logs_smoked_at;
DROP INDEX IF EXISTS idx_smoking_logs_discord_id;

DROP TABLE IF EXISTS smoking_logs;
DROP TABLE IF EXISTS smoking_types;
DROP TABLE IF EXISTS users;

