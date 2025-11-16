-- 1. Drop trigger
DROP TRIGGER IF EXISTS set_updated_at_scales ON scales;

-- 2. Drop brokers table
DROP TABLE IF EXISTS scales;
