use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoarderError {
    InvalidBorderType(String),
    InvalidEsp32Version(String),
    InvalidRaspberryPiVersion(String)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BoarderType {
    ESP32(Esp32Version),
    RaspberryPi(RaspberryPiVersion),
}

impl BoarderType{
    pub fn from_request(boarder: &str, version: &str) -> Result<Self, BoarderError> {
        match boarder.to_lowercase().as_str() {
            "esp32" => Ok(BoarderType::ESP32(version.parse()?)),
            "raspberrypi" => Ok(BoarderType::RaspberryPi(version.parse()?)),
            _ => Err(BoarderError::InvalidBorderType(format!("Invalid boarder: {}", boarder)))?
        }
    }
}

impl fmt::Display for BoarderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BoarderType::ESP32(_version) => "ESP32",
            BoarderType::RaspberryPi(_version) => "RaspberryPi",
        };
        write!(f, "{}", s)
    }
}

impl BoarderType {
    pub fn as_int(&self) -> i32 {
        match self {
            BoarderType::ESP32(_version) => 0,
            BoarderType::RaspberryPi(_version) => 1,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Esp32Version {
    S2 = 0,
    S3 = 1,
    C3 = 2,
}

impl FromStr for Esp32Version {
    type Err = BoarderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "s2" => Ok(Esp32Version::S2),
            "s3" => Ok(Esp32Version::S3),
            "c3" => Ok(Esp32Version::C3),
            _ => Err(
                BoarderError::InvalidEsp32Version(
                    format!(
                        "Invalid version for ESP32 board: Version: {}", s.to_string()
                    )
                )
            )?
        }
    }
}

impl fmt::Display for Esp32Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Esp32Version::S2 => "S2",
            Esp32Version::S3 => "S3",
            Esp32Version::C3 => "C3",
        };
        write!(f, "{}", s)
    }
}

impl Esp32Version {
    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}



#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaspberryPiVersion {
    Zero = 0,
    Three = 1,
    Four = 2,
    Five = 3,
}

impl FromStr for RaspberryPiVersion {
    type Err = BoarderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zero" => Ok(RaspberryPiVersion::Zero),
            "three" => Ok(RaspberryPiVersion::Three),
            "four" => Ok(RaspberryPiVersion::Four),
            "five" => Ok(RaspberryPiVersion::Five),
            _ => Err(
                BoarderError::InvalidRaspberryPiVersion(
                    format!(
                        "Invalid version for raspberry board: Version: {}", s.to_string()
                    )
                )
            )
        }
    }
}

impl fmt::Display for RaspberryPiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self{
            RaspberryPiVersion::Zero => "Zero",
            RaspberryPiVersion::Three => "Three",
            RaspberryPiVersion::Four => "Four",
            RaspberryPiVersion::Five => "Five",
        };
        write!(f, "{}", s)
    }
}

impl RaspberryPiVersion {
    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}