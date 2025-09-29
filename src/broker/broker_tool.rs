use actix_web::web;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;
use crate::broker::broker_model::BrokerManager;
use crate::broker::broker_query::{get_broker_with_uuid_query, put_broker_state_query};
use crate::device::device_message_model::{DeviceMessageSubscribe, SubscribeTopicQos};
use crate::error_app::error_app::{AppError, AppMsgError, AppMsgInfError};

pub fn build_subscribe_all_topics_qoss(subs: Vec<DeviceMessageSubscribe>) -> SubscribeTopicQos {
    let topics: Vec<String> = subs.iter().map(|s| s.topic.clone()).collect();
    let qoss: Vec<i32> = subs.iter().map(|s| s.qos).collect();

    SubscribeTopicQos {
        topics,
        qoss
    }
}

pub async fn broker_change_state(
    broker_uuid: Uuid,
    connected: bool,
    pool: &PgPool,
    search_broker: bool,
) -> Result<(), AppError> {

    if search_broker {
        if let Some(broker) = get_broker_with_uuid_query(pool, &broker_uuid).await? {
            if broker.connected == connected {
                return Ok(());
            }
        } else {
            return Err(AppError::NotFound(AppMsgError {
                api_msg_error: "Broker not found".to_string(),
                log_msg_error: format!("Broker not found, uuid: {}", broker_uuid),
            }));
        }
    }

    put_broker_state_query(pool, &broker_uuid, connected).await?;

    Ok(())
}

pub async fn build_subscribe_topic_qos(
    broker_uuid: Uuid,
    topic: String,
    qos: i32,
    manager: web::Data<BrokerManager>,
) -> Result<(), AppError>{

    let handle_opt = manager.get(&broker_uuid).await;

    let handle = match handle_opt {
        Some(handle) => handle,
        None => Err(AppError::NotFound(AppMsgError {
            api_msg_error: "Broker not found".to_string(),
            log_msg_error: format!("Broker not found, uuid: {}", broker_uuid),
        }))?,
    };

    let client = handle.client.clone();
    let cli = client.lock().await;
    
    cli.subscribe(&topic, qos).await.map_err(|err| {
        AppError::MqttError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "MqttError".into(),
            log_msg_error: err.to_string(),
        })
    })?;
    
    info!("📡 Subscribing broker {} to topic {}", broker_uuid, topic);

    Ok(())
}