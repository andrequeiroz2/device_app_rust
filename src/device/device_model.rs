use actix_web::web;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Device {
    pub id: i32,
    pub uuid: Uuid,
    pub user_id: i32,
    pub name: String,
    pub topic: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCreate {
    pub name: String,
    pub topic: String,
}

impl From<web::Json<DeviceCreate>> for DeviceCreate {
    fn from(device: web::Json<DeviceCreate>) -> Self {
    DeviceCreate {
        name: device.name.to_lowercase().clone(),
        topic: device.topic.clone(),
        }
    }
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct DeviceResponse {
    pub name: String,
    pub topic: String,
}

