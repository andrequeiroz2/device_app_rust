use log::info;
use crate::broker::broker_model::BrokerResponse;
use mqtt_device;
use futures::stream::StreamExt;
use tokio::time::{sleep, Duration};

const TOPICS: &[&str] = &["test/#", "hello"];
const QOS: &[i32] = &[1, 1];
pub async fn connect(
    broker: &BrokerResponse
)-> Result<(), mqtt_device::error::MqttError> {

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
    }.create_connection_options()?;

    let mut cli = match mqtt_device::create_client::create_async_client(options){
        Ok(client) => client,
        Err(err) => Err(err)?
    };

    let mut stream = cli.get_stream(25);

    if let Err(err) = cli.connect(connection_options).await {
        info!("Initial connection failed: {}", err);
        Err(mqtt_device::error::MqttError::ConnectionError(err.to_string()))?
    }

    info!("Subscribing to topics: {:?}", TOPICS);
    if let Err(err) = cli.subscribe_many(TOPICS, QOS).await {
        info!("Subscribe failed: {}", err);
        Err(mqtt_device::error::MqttError::ConnectionError(err.to_string()))?
    };

    info!("Waiting for messages...");

    let mut rconn_attempt: usize = 0;

    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            println!("📥 MQTT message received:\n{}", msg);
        } else {
            println!("Lost connection. Attempting reconnect...");
            while let Err(err) = cli.reconnect().await {
                rconn_attempt += 1;
                eprintln!("Reconnect attempt #{} failed: {}", rconn_attempt, err);
                sleep(Duration::from_secs(1)).await;
            }
            println!("✅ Reconnected.");

            //Re-subscreve após reconexão
            if let Err(err) = cli.subscribe_many(TOPICS, QOS).await {
                eprintln!("Resubscribe failed: {}", err);
                Err(mqtt_device::error::MqttError::ConnectionError(err.to_string()))?
            } else {
                println!("🔁 Resubscribed to topics: {:?}", TOPICS);
            }
        }
    }
    Ok(())
}