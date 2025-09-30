use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData {
    pub device_uuid: String,
    pub user_uuid: String,
    pub messages: Vec<DeviceMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}