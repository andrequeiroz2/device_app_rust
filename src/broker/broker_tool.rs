use sqlx::PgPool;
use uuid::Uuid;
use crate::broker::broker_query::{get_broker_with_uuid_query, put_broker_state_query};
use crate::device::device_message_model::{DeviceMessageSubscribe, SubscribeTopicQos};
use crate::error_app::error_app::{AppError, AppMsgError};

pub fn build_subscribe_topic_qos(subs: Vec<DeviceMessageSubscribe>) -> SubscribeTopicQos {
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