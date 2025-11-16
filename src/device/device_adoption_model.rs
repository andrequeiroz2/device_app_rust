use eui48::MacAddress;
use uuid::Uuid;
use crate::device::device_border_model::BoarderType;
use crate::error_app::error_app::AppError;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAdoptionRequest{
    pub board_type: String,
    pub mac_address: String,
    pub device_type: String,
    pub sensor_type: String,
    pub actuator_type: String,
    pub adopted_status: i32,
    pub device_scale: Vec<(String, String)>,
    pub device_name: String
}

impl DeviceAdoptionRequest{
    pub fn validate(&self)-> Result<(), AppError>{

        BoarderType::from_request(&self.board_type)
            .map_err(|_| {
                error!("Invalid board type: {}", &self.board_type);
                AppError::BadRequest("Invalid board type".to_string())
            })?;

        self.mac_address.parse::<MacAddress>()
            .map_err(|_| {
                error!("Invalid mac_address: {}", self.mac_address);
                AppError::BadRequest("Invalid mac_address".to_string())
            })?;

        if self.adopted_status == 1 {
            error!("device already adopted: adopted_status 1");
            Err(AppError::BadRequest("Invalid mac_address".to_string()))?
        }

        Ok(())
    }
}

