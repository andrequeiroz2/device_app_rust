use std::fmt;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoarderError {
    InvalidBorderType(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BoarderType {
    ESP32,
    RaspberryPi,
}

impl BoarderType{
    pub fn from_request(boarder: &str) -> Result<Self, BoarderError> {
        match boarder.to_lowercase().as_str() {
            "esp32" => Ok(BoarderType::ESP32),
            "raspberrypi" => Ok(BoarderType::RaspberryPi),
            _ => Err(BoarderError::InvalidBorderType(format!("Invalid boarder: {}", boarder)))?
        }
    }
}

impl fmt::Display for BoarderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BoarderType::ESP32 => "ESP32",
            BoarderType::RaspberryPi => "RaspberryPi",
        };
        write!(f, "{}", s)
    }
}

impl BoarderType {
    pub fn as_int(&self) -> i32 {
        match self {
            BoarderType::ESP32 => 0,
            BoarderType::RaspberryPi => 1,
        }
    }
}
