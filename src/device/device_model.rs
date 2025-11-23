use core::fmt;
use std::str::FromStr;
use chrono::Utc;
use actix_web::web;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::device::device_border_model::BoardType;
use crate::device::device_type_model::DeviceType;
use crate::error_app::error_app::AppError;
use eui48::MacAddress;
use crate::device::device_message_model::{DeviceMessageCreate, DeviceMessageCreateRequest, DeviceMessageCreateResponse, DeviceScaleCreate, DeviceScaleCreateResponse};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCondition {
    Adopted = 0,
    NotAdopted = 1,
    Blocked = 2,
}

impl FromStr for DeviceCondition {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "adopted" => Ok(DeviceCondition::Adopted),
            "not_adopted" => Ok(DeviceCondition::NotAdopted),
            "blocked" => Ok(DeviceCondition::Blocked),
            _ => Err(AppError::BadRequest(format!("Invalid device condition: {}", s)))?
        }
    }
}

impl fmt::Display for DeviceCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DeviceCondition::Adopted => "Adopted",
            DeviceCondition::NotAdopted => "NotAdopted",
            DeviceCondition::Blocked => "Blocked",
        };
        write!(f, "{}", s)
    }
}
impl DeviceCondition {
    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Device {
    pub id: i32,
    pub uuid: Uuid,
    pub user_id: i32,
    pub name: String,
    pub device_type_int: i32,
    pub device_type_text: String,
    pub board_type_int: i32,
    pub board_type_text: String,
    pub sensor_type: Option<String>,
    pub actuator_type: Option<String>,
    pub device_condition_int: i32,
    pub device_condition_text: String,
    pub mac_address: String,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCreateRequest{
    name: String,
    device_type_str: String,
    board_type_str: String,
    sensor_type: Option<String>,
    actuator_type: Option<String>,
    adopted_status: String,
    mac_address: String,
    message: DeviceMessageCreateRequest,
    scale: Option<Vec<(String, String)>>,
}
impl From<web::Json<DeviceCreateRequest>> for DeviceCreateRequest {
    fn from(device: web::Json<DeviceCreateRequest>) -> Self {
        let device = device.into_inner();
        DeviceCreateRequest {
            name: device.name,
            device_type_str: device.device_type_str,
            board_type_str: device.board_type_str,
            sensor_type: device.sensor_type,
            actuator_type: device.actuator_type,
            adopted_status: device.adopted_status,
            mac_address: device.mac_address,
            message: device.message,
            scale: device.scale,
        }
    }
}

impl DeviceCreateRequest {
    pub fn get_device_create_scale(&self) -> &Option<Vec<(String, String)>> {
        &self.scale
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCreate {
    pub uuid: Uuid,
    pub user_id: i32,
    pub name: String,
    pub device_type_int: i32,
    pub device_type_text: String,
    pub board_type_int: i32,
    pub board_type_text: String,
    pub mac_address: String,
    pub sensor_type: Option<String>,
    pub actuator_type: Option<String>,
    pub device_condition_int: i32,
    pub device_condition_text: String,
    pub message: DeviceMessageCreate,
    pub scale: Option<Vec<DeviceScaleCreate>>
}

impl DeviceCreate {
    pub async fn new(params: &DeviceCreateRequest, user_id: i32) -> Result<Self, AppError> {

        let uuid = Uuid::new_v4();

        match params.mac_address.parse::<MacAddress>() {
            Ok(mac) => mac,
            Err(e) => Err(AppError::BadRequest(format!("Invalid mac_address: {}", e)))?,
        };

        //device_type
        let device_type = match DeviceType::from_str(&params.device_type_str){
            Ok(device_type) => device_type,
            Err(err) => Err(AppError::BadRequest(format!("{}: {}", err, params.device_type_str)))?,
        };
        let device_type_int = device_type.as_int();
        let device_type_text = device_type.to_string();


        //border_type
        let border_type = match BoardType::from_request(&params.board_type_str){
            Ok(border_type) => border_type,
            Err(err) => Err(AppError::BadRequest(format!("{:?}", err)))?
        };
        
        let board_type_int = border_type.as_int();
        let board_type_text = border_type.to_string();

        //border_condition
        let device_condition = match DeviceCondition::from_str(&params.adopted_status){
            Ok(device_condition) => device_condition,
            Err(err) => Err(AppError::BadRequest(format!("{}", err)))?
        };

        if device_condition != DeviceCondition::Adopted{
            return Err(AppError::BadRequest("Device condition must be 'adopted'".to_string()))
        }

       match(&params.sensor_type, &params.actuator_type){
            (None, None) => {
                log::error!("file: {}, line: {}, sensor_type or actuator_type type must be specified", file!(), line!());
                Err(AppError::BadRequest("Sensor or Actuator type must be specified".to_string()))?
            }

            (Some(_), Some(_)) => {
                log::error!(
                    "file: {}, line: {}, sensor_type and actuator_type type must be specified only one: sensor_type: {:#?}, actuator_type: {:#?}", file!(), line!(), &params.sensor_type, &params.actuator_type);
                Err(AppError::BadRequest("Sensor or Actuator type must be specified".to_string()))?
            }

            _ => {},
        };
        
        let message = DeviceMessageCreate::new(&params.message)?;

        let mut scale: Option<Vec<DeviceScaleCreate>> = None;

        if let Some(scale_param) = &params.scale {
            scale = Some(DeviceScaleCreate::from_request(&params));
        };

        //variables
        let device_condition_int = device_condition.as_int();
        let device_condition_text = device_condition.to_string();
        let name = params.name.clone();
        let mac_address = params.mac_address.clone();
        let sensor_type = params.sensor_type.clone();
        let actuator_type = params.actuator_type.clone();

        Ok(
            Self {
                uuid,
                user_id,
                name,
                device_type_int,
                device_type_text,
                board_type_int,
                board_type_text,
                sensor_type,
                actuator_type,
                device_condition_int,
                device_condition_text,
                mac_address,
                message,
                scale
            }
        )
    }
    
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn get_user_id(&self) -> i32 {
        self.user_id
    }
    
    pub fn get_device_type_int(&self) -> i32 {
        self.device_type_int
    }
    
    pub fn get_device_type_text(&self) -> String {
        self.device_type_text.clone()
    }
    
    pub fn get_border_type_int(&self) -> i32 {
        self.board_type_int
    }
    
    pub fn get_border_type_text(&self) -> String {
        self.board_type_text.clone()
    }

    pub fn get_sensor_type(&self) -> Option<String> { self.sensor_type.clone() }

    pub fn get_actuator_type(&self) -> Option<String> { self.actuator_type.clone() }
    
    pub fn get_device_condition_int(&self) -> i32 {
        self.device_condition_int
    }
    
    pub fn get_device_condition_text(&self) -> String {
        self.device_condition_text.clone()
    }
    
    pub fn get_mac_address(&self) -> String {
        self.mac_address.clone()
    }
}


#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct DeviceCreateResponse {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub name: String,
    pub device_type_int: i32,
    pub device_type_text: String,
    pub board_type_int: i32,
    pub board_type_text: String,
    pub mac_address: String,
    pub device_condition_int: i32,
    pub device_condition_text: String,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub message: DeviceMessageCreateResponse,
    pub scale: Vec<DeviceScaleCreateResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceFilter{
    pub uuid: Option<Uuid>,
    pub mac_address: Option<String>,
}

