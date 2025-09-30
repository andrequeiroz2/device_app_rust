use std::sync::{Arc};
use tokio::sync::Mutex;
use actix_web::web;
use log::{info};
use crate::broker::broker_model::{BrokerHandle, BrokerManager, BrokerResponse};
use mqtt_device;
use futures::stream::StreamExt;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use crate::broker::broker_tool::{broker_change_state, build_subscribe_all_topics_qoss};
use crate::device::device_message_query::get_device_message_subscribe_query;
use crate::error_app::error_app::{AppError, AppMsgInfError};


pub async fn connect(
    pool: &PgPool,
    broker: &BrokerResponse,
    manager: web::Data<BrokerManager>,
    // manager: Arc<BrokerManager>,
)-> Result<(), AppError> {

    let options = mqtt_device::create_options::Options{
        server_host: broker.host.clone(),
        server_port: broker.port as u32,
        client_id: broker.client_id.clone(),
        version: broker.version,
    };

    let last_will = mqtt_device::components::will::Will{
        topic: broker.last_will_topic.clone().unwrap(),
        payload: broker.last_will_message.clone().unwrap(),
        qos: broker.last_will_qos.clone().unwrap(),
        retained: broker.last_will_retain.clone().unwrap(),
    };

    let connection_options = mqtt_device::create_connection_options::ConnectionOptions{
        keep_alive: broker.keep_alive as u64,
        clean_session: broker.clean_session,
        will: last_will
    }.create_connection_options()
        .map_err(|err| AppError::MqttError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "MqttError".into(),
            log_msg_error: err.to_string(),
        }))?;

    let cli = mqtt_device::create_client::create_async_client(options)
        .map_err(|err| AppError::MqttError(AppMsgInfError {
        file: file!().to_string(),
        line: line!(),
        api_msg_error: "MqttError".into(),
        log_msg_error: err.to_string(),
    }))?;

    cli.connect(connection_options).await.map_err(|err| {
        AppError::MqttError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "Connect failure".into(),
            log_msg_error: err.to_string(),
        })
    })?;

    info!("connection established");

    let subscribers = get_device_message_subscribe_query(pool).await?;

    info!("Subscribers: {:?}", subscribers);

    let subs = build_subscribe_all_topics_qoss(subscribers.clone());

    info!("Subs: {:?}", subs);

    if !subscribers.is_empty() {
        cli.subscribe_many(&subs.topics, &subs.qoss).await.map_err(|err| {
            AppError::MqttError(AppMsgInfError {
                file: file!().to_string(),
                line: line!(),
                api_msg_error: "MqttError".into(),
                log_msg_error: err.to_string(),
            })
        })?;
    }

    info!("subscribe topics successful");

    let cancel_token = CancellationToken::new();
    let cancel_child = cancel_token.child_token();

    let client = Arc::new(Mutex::new(cli));
    let handle = BrokerHandle {
        cancel_token: cancel_token.clone(),
        client: client.clone(),
    };

    manager.insert(broker.uuid, handle).await;
    info!("🔗 Broker {} inserted into manager", broker.uuid);

    let pool = pool.clone();
    let broker_uuid = broker.uuid;
    let subscribers = subscribers.clone();
    let subs = subs.clone();

    tokio::spawn({
        let manager = manager.clone();
        let client = client.clone();
        let pool = pool.clone();
        let broker_uuid = broker_uuid;
        let subscribers = subscribers.clone();
        let subs = subs.clone();
        let cancel_child = cancel_child.clone();

        async move {
            let mut cli = client.lock().await;
            let mut stream = cli.get_stream(25);
            let mut reconnect_attempt = 0;

            loop {
                tokio::select! {
                    _ = cancel_child.cancelled() => {
                        info!("🛑 Cancel received, disconnecting broker {}", broker_uuid);
                        if let Err(err) = cli.disconnect(None).await {
                            info!("Error while disconnecting: {}", err);
                        }
                        let _ = broker_change_state(broker_uuid, false, &pool, true).await;
                        break;
                    }

                    msg_opt = stream.next() => {
                        match msg_opt {
                            Some(Some(msg)) => {
                                info!("📥 MQTT message received: {}", msg);
                            }
                            Some(None) => {
                                info!("Lost connection. Attempting reconnect...");
                                while let Err(err) = cli.reconnect().await {
                                    reconnect_attempt += 1;
                                    info!("Reconnect attempt #{} failed: {}", reconnect_attempt, err);
                                    sleep(Duration::from_secs(1)).await;
                                    let _ = broker_change_state(broker_uuid, false, &pool, true).await;
                                }

                                let _ = broker_change_state(broker_uuid, true, &pool, true).await;
                                info!("✅ Reconnected.");

                                if !subscribers.is_empty() {
                                    if let Err(err) = cli.subscribe_many(&subs.topics, &subs.qoss).await {
                                        info!("Resubscribe failed: {}", err);
                                    } else {
                                        info!("🔁 Resubscribed ok");
                                    }
                                }
                            }
                            None => {
                                info!("MQTT stream closed for broker {}", broker_uuid);
                                break;
                            }
                        }
                    }
                }
            }

            // 🔴 Cleanup no final da task
            info!("🧹 Cleaning up broker {}", broker_uuid);
            manager.remove(&broker_uuid).await;
        }
    });

    Ok(())
}