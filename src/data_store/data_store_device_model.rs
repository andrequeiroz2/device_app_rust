use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use mongodb::bson::DateTime as BsonDateTime;
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Received,
    Sent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceMessage {
    pub direction: MessageDirection,
    pub topic: String,
    pub qos: i32,
    pub payload: String,
    pub timestamp: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData {
    #[serde(rename = "_id")]
    pub id: String,
    pub device_uuid: String,
    pub user_uuid: String,
    pub messages: Vec<DeviceMessage>,
    pub created_at: BsonDateTime,
    pub updated_at: Option<BsonDateTime>,
    pub deleted_at: Option<BsonDateTime>,
}