-- 1. create message table
CREATE TABLE messages (
     id SERIAL PRIMARY KEY,
     uuid UUID NOT NULL UNIQUE,
     device_id INT NOT NULL REFERENCES devices(id),
     topic VARCHAR(255) NOT NULL,
     payload VARCHAR(255) NOT NULL,
     qos INTEGER CHECK (qos IN (0, 1, 2)) NOT NULL,
     retained BOOLEAN NOT NULL,
     publisher BOOLEAN,
     subscriber BOOLEAN,
     scale VARCHAR(10),
     command_start INTEGER,
     command_end INTEGER,
     command_last INTEGER,
     command_last_time TIMESTAMPTZ,
     created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
     updated_at TIMESTAMPTZ,
     deleted_at TIMESTAMPTZ
);

-- 2. Trigger update updated_at
CREATE TRIGGER set_updated_at_messages
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
