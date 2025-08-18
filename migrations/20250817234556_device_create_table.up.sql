-- Add up migration script here
CREATE TABLE devices (
   id SERIAL PRIMARY KEY,
   uuid UUID NOT NULL UNIQUE,
   name VARCHAR(255) NOT NULL,
   topic VARCHAR(255) NOT NULL UNIQUE,
   created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
   updated_at TIMESTAMPTZ,
   deleted_at TIMESTAMPTZ
);

-- 2. Create trigger
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 3. Trigger update updated_at
CREATE TRIGGER set_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();