-- Remove trigger
DROP TRIGGER IF EXISTS set_updated_at_devices ON devices;

-- -- Drop devices table
DROP TABLE IF EXISTS devices;
