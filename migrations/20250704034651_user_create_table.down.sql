-- 1. Drop indexes
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_uuid;

-- 2. Drop trigger
DROP TRIGGER IF EXISTS set_updated_at ON users;

-- 3. Drop trigger function
DROP FUNCTION IF EXISTS trigger_set_updated_at;

-- 4. Drop users table
DROP TABLE IF EXISTS users;
