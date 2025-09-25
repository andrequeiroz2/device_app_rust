use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Sensor = 0,
    Actuator = 1,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DeviceType::Sensor => "Sensor",
            DeviceType::Actuator => "Actuator",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DeviceType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sensor" => Ok(DeviceType::Sensor),
            "actuator" => Ok(DeviceType::Actuator),
            _ => Err("Invalid device type"),
        }
    }
}

impl DeviceType {
    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}


