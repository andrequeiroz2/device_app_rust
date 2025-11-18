-- 1. create device table
CREATE TABLE devices (
   id SERIAL PRIMARY KEY,
   uuid UUID NOT NULL UNIQUE,
   user_id INT NOT NULL REFERENCES users(id),
   name VARCHAR(50) NOT NULL,
   device_type_int INT NOT NULL,
   device_type_text VARCHAR(50) NOT NULL,
   border_type_int INT NOT NULL ,
   border_type_text VARCHAR(50) NOT NULL,
   sensor_type VARCHAR(50),
   actuator_type VARCHAR(50),
   device_condition_int INT NOT NULL ,
   device_condition_text VARCHAR(50) NOT NULL,
   mac_address VARCHAR(50) NOT NULL UNIQUE,
   created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
   updated_at TIMESTAMPTZ,
   deleted_at TIMESTAMPTZ
);

-- 2. Trigger update updated_at
CREATE TRIGGER set_updated_at_devices
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();