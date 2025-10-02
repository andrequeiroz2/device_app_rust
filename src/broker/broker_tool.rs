use actix_web::web;
use log::info;
use mqtt_device::AsyncClient;
use mqtt_device::components::will::Will;
use mqtt_device::create_connection_options::ConnectionOptions;
use mqtt_device::create_options::Options;
use sqlx::PgPool;
use uuid::Uuid;
use crate::broker::broker_model::{BrokerCommand, BrokerManager, BrokerResponse};
use crate::broker::broker_query::{get_broker_with_uuid_query, put_broker_state_query};
use crate::device::device_message_model::{DeviceMessageSubscribe, SubscribeTopicQos};
use crate::error_app::error_app::{AppError, AppMsgError, AppMsgInfError};

pub fn create_options(broker: &BrokerResponse) -> Options {
    Options{
        server_host: broker.host.clone(),
        server_port: broker.port as u32,
        client_id: broker.client_id.clone(),
        version: broker.version,
    }
}

pub fn create_last_will(broker: &BrokerResponse) -> Will {
    Will{
        topic: broker.last_will_topic.clone().unwrap(),
        payload: broker.last_will_message.clone().unwrap(),
        qos: broker.last_will_qos.clone().unwrap(),
        retained: broker.last_will_retain.clone().unwrap(),
    }
}

pub async fn create_connection_options(
    broker: &BrokerResponse
) -> Result<paho_mqtt::ConnectOptions, AppError> {

    let last_will = create_last_will(broker);

    let connection_options = ConnectionOptions{
        keep_alive: broker.keep_alive as u64,
        clean_session: broker.clean_session,
        will: last_will
    };

    let connection_options = connection_options.create_connection_options()
        .map_err(|err| AppError::MqttError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "MqttError".into(),
            log_msg_error: err.to_string(),
        }))?;

    Ok(connection_options)
}

pub async fn create_client(options: Options) -> Result<AsyncClient, AppError> {

    let cli = mqtt_device::create_client::create_async_client(options)
        .map_err(|err| AppError::MqttError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "MqttError".into(),
            log_msg_error: err.to_string(),
        }))?;

    Ok(cli)

}

pub fn build_subscribe_all_topics_qoss(subs: Vec<DeviceMessageSubscribe>) -> SubscribeTopicQos {
    let topics: Vec<String> = subs.iter().map(|s| s.topic.clone()).collect();
    let qoss: Vec<i32> = subs.iter().map(|s| s.qos).collect();

    SubscribeTopicQos {
        topics,
        qoss
    }
}

pub async fn broker_change_state(
    broker_uuid: &Uuid,
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
            log_msg_error: format!(
                "file: {}: line: {}, Broker not found, uuid: {}",
                file!(),
                line!(),
                broker_uuid
            ),
        }))?,
    };

    handle.cmd_tx.send(BrokerCommand::Subscribe{topic, qos})
        .await
        .map_err(|err|
            AppError::MqttError(AppMsgInfError {
                file: file!().to_string(),
                line: line!(),
                api_msg_error: "MqttError".into(),
                log_msg_error: err.to_string(),
            }))?;

    Ok(())
}