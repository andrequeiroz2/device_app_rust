-- 1. Drop trigger
DROP TRIGGER IF EXISTS set_updated_at_messages ON messages;

-- 2. Drop brokers table
DROP TABLE IF EXISTS messages;
