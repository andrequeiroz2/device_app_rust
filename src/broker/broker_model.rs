use actix_web::web;
use actix_web::web::Query;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LastWill{
    pub topic: String,
    pub payload: String,
    pub qos: i32,
    pub retained: bool,
}

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Broker {
    pub id: i32,
    pub uuid: Uuid,
    pub server_host: String,
    pub server_port: i32,
    pub server_id: String,
    pub server_version: i32,
    pub keep_alive: i32,
    pub clean_session: bool,
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub last_will_qos: Option<i32>,
    pub last_will_retain: Option<bool>,
    pub connected: Option<bool>,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrokerCreate {
    pub host: String,
    pub port: i32,
    pub client_id: String,
    pub version: i32,
    pub keep_alive: i32,
    pub clean_session: bool,
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub last_will_qos: Option<i32>,
    pub last_will_retain: Option<bool>,
}

impl From<web::Json<BrokerCreate>> for BrokerCreate {
    fn from(broker: web::Json<BrokerCreate>) -> Self {

        BrokerCreate{
            host: broker.host.clone(),
            port: broker.port,
            client_id: broker.client_id.clone(),
            version: broker.version,
            keep_alive: broker.keep_alive,
            clean_session: broker.clean_session,
            last_will_topic: broker.last_will_topic.clone(),
            last_will_message: broker.last_will_message.clone(),
            last_will_qos: broker.last_will_qos.clone(),
            last_will_retain: broker.last_will_retain.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct BrokerResponse {
    pub uuid: Uuid,
    pub host: String,
    pub port: i32,
    pub client_id: String,
    pub version: i32,
    pub version_text: String,
    pub keep_alive: i32,
    pub clean_session: bool,
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub last_will_qos: Option<i32>,
    pub last_will_retain: Option<bool>,
    pub connected: bool,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrokerFilter {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub connected: Option<bool>,
}

impl From<web::Query<BrokerFilter>> for BrokerFilter {
    fn from(filter: Query<BrokerFilter>) -> Self {
        BrokerFilter{
            id: filter.id,
            uuid: filter.uuid,
            host: filter.host.clone(),
            port: filter.port,
            connected: filter.connected,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrokerUpdate {
    pub host: String,
    pub port: i32,
    pub client_id: String,
    pub version: i32,
    pub keep_alive: i32,
    pub clean_session: bool,
    pub last_will_topic: String,
    pub last_will_message: String,
    pub last_will_qos: i32,
    pub last_will_retain: bool,
    pub connected: bool,
}

impl From<web::Json<BrokerUpdate>> for BrokerUpdate {
    fn from(broker: web::Json<BrokerUpdate>) -> Self {
        BrokerUpdate{
            host: broker.host.clone(),
            port: broker.port,
            client_id: broker.client_id.clone(),
            version: broker.version,
            keep_alive: broker.keep_alive,
            clean_session: broker.clean_session,
            last_will_topic: broker.last_will_topic.clone(),
            last_will_message: broker.last_will_message.clone(),
            last_will_qos: broker.last_will_qos.clone(),
            last_will_retain: broker.last_will_retain.clone(),
            connected: broker.connected.clone(),
        }
    }
}
