use std::sync::Arc;
use actix_web::web;
use log::info;
use crate::broker::broker_model::{BrokerCommand, BrokerHandle, BrokerManager, BrokerResponse};
use futures::stream::StreamExt;
use mongodb::Client;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use crate::broker::broker_tool::{broker_change_state, build_subscribe_all_topics_qoss, create_client, create_connection_options, create_options};
use crate::data_store::data_store_device_handler::put_device_collection;
use crate::device::device_message_query::get_device_message_subscribe_query;
use crate::error_app::error_app::{AppError, AppMsgInfError};


pub async fn connect(
    pool: &PgPool,
    mongo_db: Client,
    broker: &BrokerResponse,
    manager: web::Data<BrokerManager>,
)-> Result<(), AppError> {

    let options = create_options(&broker);
    let connection_options = create_connection_options(&broker).await?;
    let mut cli = create_client(options).await?;
    let mut stream = cli.get_stream(25);

    cli.connect(connection_options).await.map_err(|err| {
        AppError::MqttError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "Mqtt client connection failure".into(),
            log_msg_error: err.to_string(),
        })
    })?;

    info!("file: {}, line: {}: Mqtt client connection established",
        file!(),
        line!()
    );

    let subscribers = get_device_message_subscribe_query(pool).await?;
    let subs = build_subscribe_all_topics_qoss(subscribers.clone());

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

    let cancel_token = CancellationToken::new();
    let cancel_child = cancel_token.child_token();
    let (cmd_tx, cmd_rx) = mpsc::channel::<BrokerCommand>(32);

    let client = Arc::new(cli);
    let handle = BrokerHandle {
        cancel_token: cancel_token.clone(),
        client: client.clone(),
        cmd_tx,
    };

    manager.insert(broker.uuid, handle).await;
    info!("file: {}, line: {}: Broker {} inserted into manager",
        file!(),
        line!(),
        broker.uuid
    );

    let broker_uuid = broker.uuid;

    tokio::spawn({
        let manager = manager.clone();
        let client = client.clone();
        let pool = pool.clone();
        let broker_uuid = broker_uuid;
        let subscribers = subscribers.clone();
        let subs = subs.clone();
        let cancel_child = cancel_child.clone();
        let mut cmd_rx = cmd_rx;
        let mongo_db = mongo_db;

        async move {
            let mut reconnect_attempt = 0;

            loop {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        BrokerCommand::Subscribe { topic, qos } => {
                            if let Err(err) = client.subscribe(&topic, qos).await {
                                info!("file: {}, line: {}: Failed to subscribe: {}, topic: {}, qos: {}",
                                   file!(),
                                   line!(),
                                   topic,
                                   qos,
                                   err
                                );
                            } else {
                                info!("Subscribed to topic: {}, qos: {}", topic, qos);
                            }
                        }
                        BrokerCommand::Unsubscribe { topic } => {
                            if let Err(err) = client.unsubscribe(&topic).await {
                                info!("file: {}, line: {}: Failed to unsubscribe: {}: topic: {}",
                                   file!(),
                                   line!(),
                                   err,
                                   topic,
                                );
                            } else {
                                info!("Unsubscribed from {}", topic);
                            }
                        }
                    }
                }

                _ = cancel_child.cancelled() => {
                    info!("file: {}, line: {}: Request disconnecting broker {}",
                        file!(),
                        line!(),
                        broker_uuid
                    );

                    if let Err(err) = client.disconnect(None).await {
                        info!("file: {}, line: {}: Error while disconnecting: {}",
                            file!(),
                            line!(),
                            err
                        );
                    }
                    // let _ = broker_change_state(broker_uuid, false, &pool, true).await;
                    break;
                }

                    msg_opt = stream.next() => {
                        match msg_opt {
                            Some(Some(msg)) => {
                                info!("📥 MQTT message received: {}", msg);
                                put_device_collection(mongo_db.clone(), &msg).await;
                            }
                            Some(None) => {
                                info!("Lost connection. Attempting reconnect...");
                                while let Err(err) = client.reconnect().await {
                                    reconnect_attempt += 1;
                                    info!("Reconnect attempt #{} failed: {}", reconnect_attempt, err);
                                    sleep(Duration::from_secs(1)).await;
                                    let _ = broker_change_state(&broker_uuid, false, &pool, true).await;
                                }

                                let _ = broker_change_state(&broker_uuid, true, &pool, true).await;
                                info!("✅ Reconnected.");

                                if !subscribers.is_empty() {
                                    if let Err(err) = client.subscribe_many(&subs.topics, &subs.qoss).await {
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