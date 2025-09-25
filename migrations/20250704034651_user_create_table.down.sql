-- 1. Drop trigger
DROP TRIGGER IF EXISTS set_updated_at_users ON users;

-- 2. Drop trigger function
DROP FUNCTION IF EXISTS trigger_set_updated_at;

-- 3. Drop users table
DROP TABLE IF EXISTS users;
