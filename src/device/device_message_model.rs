use actix_web::web;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use mqtt_device;
use crate::error_app::error_app::{AppError};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct DeviceMessage {
    pub id: i32,
    pub uuid: Uuid,
    pub device_id: i32,
    pub topic: String,
    pub payload: String,
    pub qos: i32,
    pub retained: bool,
    pub publisher: Option<bool>,
    pub subscriber: Option<bool>,
    pub scale: Option<String>,
    pub command_start: Option<i32>,
    pub command_end: Option<i32>,
    pub command_last: Option<i32>,
    pub command_last_time: Option<chrono::DateTime<Utc>>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceMessageCreateRequest {
    // device_id: i32,
    topic: String,
    payload: String,
    qos: i32,
    retained: bool,
    publisher: Option<bool>,
    subscriber: Option<bool>,
    scale: Option<String>,
    command_start: Option<i32>,
    command_end: Option<i32>,
}

impl From<web::Json<DeviceMessageCreateRequest>> for DeviceMessageCreateRequest {
    fn from(message: web::Json<DeviceMessageCreateRequest>) -> Self {
        let message = message.into_inner();
        DeviceMessageCreateRequest{
            // device_id: message.device_id,
            topic: message.topic,
            payload: message.payload,
            qos: message.qos,
            retained: message.retained,
            publisher: message.publisher,
            subscriber: message.subscriber,
            scale: message.scale,
            command_start: message.command_start,
            command_end: message.command_end,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceMessageCreate {
    pub uuid: Uuid,
    // pub device_id: i32,
    pub topic: String,
    pub payload: String,
    pub qos: i32,
    pub retained: bool,
    pub publisher: Option<bool>,
    pub subscriber: Option<bool>,
    pub scale: Option<String>,
    pub command_start: Option<i32>,
    pub command_end: Option<i32>,
    pub command_last: Option<i32>,
    pub command_last_time: Option<chrono::DateTime<Utc>>,
}

impl DeviceMessageCreate{
    pub fn new(params: DeviceMessageCreateRequest)-> Result<DeviceMessageCreate, AppError>{
        let uuid = Uuid::new_v4();

        mqtt_device::components::topic::valid_topic(&params.topic)
            .map_err(|err| AppError::BadRequest(err.to_string()))?;

        mqtt_device::components::payload::validate_payload_size(&params.payload)
            .map_err(|err| AppError::BadRequest(err.to_string()))?;

        mqtt_device::components::qos::Qos::valid_qos(params.qos)
            .map_err(|err| AppError::BadRequest(err.to_string()))?;
        
        Ok(
            DeviceMessageCreate{
                uuid,
                // device_id: params.device_id,
                topic: params.topic,
                payload: params.payload,
                qos: params.qos,
                retained: params.retained,
                publisher: Some(params.publisher.unwrap_or(false)),
                subscriber: Some(params.subscriber.unwrap_or(false)),
                scale: params.scale,
                command_start: params.command_start,
                command_end: params.command_end,
                command_last: None,
                command_last_time: None,
            }
        )
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
    // pub fn get_device_id(&self) -> i32 {
    //     self.device_id
    // }
    pub fn get_topic(&self) -> String {
        self.topic.clone()
    }
    pub fn get_payload(&self) -> String {
        self.payload.clone()
    }
    pub fn get_qos(&self) -> i32 {
        self.qos
    }
    pub fn get_retained(&self) -> bool {
        self.retained
    }
    pub fn get_publisher(&self) -> Option<bool> {
        self.publisher
    }
    pub fn get_subscriber(&self) -> Option<bool> {
        self.subscriber
    }
    pub fn get_scale(&self) -> Option<String> {
        self.scale.clone()
    }
    pub fn get_command_start(&self) -> Option<i32> {
        self.command_start
    }
    pub fn get_command_end(&self) -> Option<i32> {
        self.command_end
    }
    pub fn get_command_last(&self) -> Option<i32> {
        self.command_last
    }
    pub fn get_command_last_time(&self) -> Option<chrono::DateTime<Utc>> {
        self.command_last_time
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceMessageCreateResponse {
    pub uuid: Uuid,
    pub device_uuid: Uuid,
    pub topic: String,
    pub payload: String,
    pub qos: i32,
    pub retained: bool,
    pub publisher: Option<bool>,
    pub subscriber: Option<bool>,
    pub scale: Option<String>,
    pub command_start: Option<i32>,
    pub command_end: Option<i32>,
    pub command_last: Option<i32>,
    pub command_last_time: Option<chrono::DateTime<Utc>>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceMessageSubscribe {
    pub device_uuid: Uuid,
    pub message_uuid: Uuid,
    pub topic: String,
    pub qos: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeTopicQos {
    pub topics: Vec<String>,
    pub qoss: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MessageReceivePayload {
    pub topic: String,
    pub device_uuid: String,
    pub user_uuid: String,
    pub payload: String,
}