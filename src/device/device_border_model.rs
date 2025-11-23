use std::fmt;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoarderError {
    InvalidBoardType(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BoardType {
    ESP32,
    RaspberryPi,
}

impl BoardType{
    pub fn from_request(boarder: &str) -> Result<Self, BoarderError> {
        match boarder.to_lowercase().as_str() {
            "esp32" => Ok(BoardType::ESP32),
            "raspberrypi" => Ok(BoardType::RaspberryPi),
            _ => Err(BoarderError::InvalidBoardType(format!("Invalid boarder: {}", boarder)))?
        }
    }
}

impl fmt::Display for BoardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BoardType::ESP32 => "ESP32",
            BoardType::RaspberryPi => "RaspberryPi",
        };
        write!(f, "{}", s)
    }
}

impl BoardType {
    pub fn as_int(&self) -> i32 {
        match self {
            BoardType::ESP32 => 0,
            BoardType::RaspberryPi => 1,
        }
    }
}
