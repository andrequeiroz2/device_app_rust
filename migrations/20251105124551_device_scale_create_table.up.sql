
CREATE TABLE scales (
                          id SERIAL PRIMARY KEY,
                          uuid UUID NOT NULL UNIQUE,
                          device_id INT NOT NULL REFERENCES devices(id),
                          metric VARCHAR(255) NOT NULL,
                          unit VARCHAR(255) NOT NULL,
                          created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                          updated_at TIMESTAMPTZ,
                          deleted_at TIMESTAMPTZ
);

-- 2. Trigger update updated_at
CREATE TRIGGER set_updated_at_scales
    BEFORE UPDATE ON scales
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
