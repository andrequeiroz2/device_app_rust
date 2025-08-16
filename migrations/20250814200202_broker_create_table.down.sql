-- 1. Drop indexes
DROP INDEX IF EXISTS idx_brokers_host;
DROP INDEX IF EXISTS idx_brokers_port;
DROP INDEX IF EXISTS idx_brokers_uuid;

-- 2. Drop trigger
DROP TRIGGER IF EXISTS set_updated_at_brokers ON brokers;

-- 3. Drop brokers table
DROP TABLE IF EXISTS brokers;
