use chrono::Utc;
use serde::{Deserialize, Serialize};
use mongodb::bson::{DateTime as BsonDateTime};
use uuid::Uuid;


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
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData {
    #[serde(rename = "_id")]
    pub id: String,
    pub device_uuid: String,
    pub user_uuid: String,
    #[serde(default)]
    pub messages: Vec<DeviceMessage>,
    pub created_at: BsonDateTime,
    pub updated_at: Option<BsonDateTime>,
    pub deleted_at: Option<BsonDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceDataStoreResponse {
    pub id: Uuid,
    pub device_uuid: Uuid,
    pub user_uuid: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}
