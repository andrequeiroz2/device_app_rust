-- 1. Drop trigger
DROP TRIGGER IF EXISTS set_updated_at_brokers ON brokers;

-- 2. Drop brokers table
DROP TABLE IF EXISTS brokers;
