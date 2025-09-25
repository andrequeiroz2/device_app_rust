-- 1. create brokers table
CREATE TABLE brokers (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL UNIQUE,
    client_id VARCHAR(255) NOT NULL,
    version INTEGER NOT NULL CHECK (version IN (0, 3, 4, 5)),
    version_text TEXT GENERATED ALWAYS AS (
        CASE version
            WHEN 0 THEN 'default'
            WHEN 3 THEN 'v3_1'
            WHEN 4 THEN 'v3_1_1'
            WHEN 5 THEN 'v5'
            END
        ) STORED,
    keep_alive INTEGER NOT NULL,
    clean_session BOOLEAN NOT NULL,
    last_will_topic VARCHAR(255),
    last_will_message VARCHAR(255),
    last_will_qos INTEGER CHECK (last_will_qos IN (0, 1, 2)),
    last_will_retain BOOLEAN,
    connected BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

-- 2. Trigger update updated_at
CREATE TRIGGER set_updated_at_brokers
    BEFORE UPDATE ON brokers
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
