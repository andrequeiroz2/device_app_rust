use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use actix_web::web;
use actix_web::web::Query;
use chrono::Utc;
use mqtt_device::AsyncClient;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio_util::sync::CancellationToken;
use crate::paginate::paginate_model::{Pagination, PaginationFrom};

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

#[derive(Serialize)]
pub struct BrokerPaginationResponse {
    pub brokers: Vec<BrokerResponse>,
    pub pagination: PaginationFrom,
    pub total_count: i64,
    pub total_pages: u32,
    pub current_page: u32,
    pub next_page: Option<i64>,
    pub previous_page: Option<i64>,
    pub first_page: u32,
    pub last_page: u32,
    pub has_next_page: bool,
}

impl BrokerPaginationResponse {
    pub fn new(
        brokers: Vec<BrokerResponse>,
        total_count: i64,
        page: u32,
        page_size: u32,
    ) -> Self {
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as u32;

        let current_page = page.max(1).min(total_pages.max(1));

        let next_page = if current_page < total_pages {
            Some((current_page + 1) as i64)
        } else {
            None
        };

        let previous_page = if current_page > 1 {
            Some((current_page - 1) as i64)
        } else {
            None
        };

        Self {
            brokers,
            pagination: PaginationFrom{ page: current_page, page_size },
            total_count,
            total_pages,
            current_page,
            next_page,
            previous_page,
            first_page: 1,
            last_page: total_pages.max(1),
            has_next_page: next_page.is_some(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrokerFilter {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub connected: Option<bool>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

impl From<web::Query<BrokerFilter>> for BrokerFilter {
    fn from(filter: Query<BrokerFilter>) -> Self {

        BrokerFilter{
            id: filter.id,
            uuid: filter.uuid,
            host: filter.host.clone(),
            port: filter.port,
            connected: filter.connected,
            pagination: filter.pagination.clone(),
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

#[derive(Clone)]
pub struct BrokerHandle {
    pub cancel_token: CancellationToken,
    pub client: Arc<Mutex<AsyncClient>>,
}

#[derive(Clone, Default)]
pub struct BrokerManager{
    brokers: Arc<Mutex<HashMap<Uuid, BrokerHandle>>>,
}

impl BrokerManager{
    pub async fn insert(&self, broker_uuid: Uuid, handle: BrokerHandle) {
        let mut brokers = self.brokers.lock().await;
        brokers.insert(broker_uuid, handle);
        log::info!("🔗 Inserted broker {} into manager (total: {})", broker_uuid, brokers.len());
    }

    pub async fn get(&self, broker_uuid: &Uuid) -> Option<BrokerHandle> {
        let brokers = self.brokers.lock().await;
        log::info!("🔍 Get broker {} (total: {})", broker_uuid, brokers.len());
        brokers.get(broker_uuid).cloned()
    }

    pub async fn remove(&self, broker_uuid: &Uuid) {
        let mut brokers = self.brokers.lock().await;
        brokers.remove(broker_uuid);
    }

    // pub async fn subscribe_new_topic(
    //     &self,
    //     broker_uuid: &Uuid,
    //     topic: &str,
    //     qos: i32
    // ) -> Result<(), mqtt_device::Error> {
    //     if let Some(handle) = self.get(broker_uuid).await {
    //         let mut client = handle.client.lock().await;
    //         client.subscribe(topic, qos).await?;
    //     }
    //     Ok(())
    // }
}