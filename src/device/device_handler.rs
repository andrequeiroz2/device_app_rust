use actix_web::{web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use log::{info, warn};
use web::Json;
use crate::auth::auth_tool::token_info;
use crate::broker::broker_model::{BrokerManager};
use crate::broker::broker_query::{get_broker_connected_query};
use crate::broker::broker_tool::build_subscribe_topic_qos;
use crate::data_store::data_store_device_handler::create_device_collection;
use crate::device::device_adoption_tool::device_compose_topic;
use crate::device::device_model::{DeviceCreate, DeviceCreateRequest, DeviceCreateResponse, DeviceFilter};
use crate::device::device_query::{get_device_filter, post_device_message_query};
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::state::AppState;
use crate::user::user_query::get_user_by_uuid;
use crate::device::device_message_model::{DeviceMessageCreateResponse, DeviceScale, DeviceScaleCreateResponse};

pub async fn device_create(
    device: Json<DeviceCreateRequest>,
    credentials: BearerAuth,
    app_state: web::Data<AppState>,
    manager: web::Data<BrokerManager>
)-> Result<HttpResponse, AppError>{

    let token = token_info(credentials.token().to_string()).await?;

    let user = get_user_by_uuid(&app_state.db, &token.inf.uuid).await?;

    let device = device.into_inner();
    let device = DeviceCreate::new(&device, user.id).await?;

    let device_filter = DeviceFilter{
        uuid: None,
        mac_address: Some(device.mac_address.clone()),
    };

    let device_check = get_device_filter(&app_state.db, &device_filter).await?;

    if let Some(value) = device_check {
        return Err(
            AppError::ConstraintViolation(
                AppMsgError {
                    api_msg_error: "Device already registered".into(),
                    log_msg_error: format!("Device already registered: {:?}", value),
                }
            )
        )?
    };

    let broker = get_broker_connected_query(&app_state.db).await?;
    if !broker.is_some(){
        return Err(
            AppError::NotFound(
                AppMsgError {
                    api_msg_error: "Broker not connected. Connect to an MQTT broker before registering a device".into(),
                    log_msg_error: "Broker not connected. Connect to an MQTT broker before registering a device".into(),
                }
            )
        )
    }

    let topic_compose = device_compose_topic(&user.uuid, &device.uuid, &device.name);

    mqtt_device::components::topic::valid_topic(&topic_compose)
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let (result_device, result_message, result_scale) = post_device_message_query(&app_state.db, &device, topic_compose.clone()).await?;
    
    if broker.is_some() && device.device_type_int == 0 {
        let broker = broker.unwrap();
        let _ = build_subscribe_topic_qos(broker.uuid, topic_compose, device.message.qos,  manager.clone()).await?;
    }

    let result = DeviceCreateResponse{
        uuid: result_device.uuid,
        user_uuid: user.uuid,
        name: result_device.name,
        device_type_int: result_device.device_type_int,
        device_type_text: result_device.device_type_text,
        board_type_int: result_device.board_type_int,
        board_type_text: result_device.board_type_text,
        mac_address: result_device.mac_address,
        device_condition_int: result_device.device_condition_int,
        device_condition_text: result_device.device_condition_text,
        created_at: result_device.created_at,
        updated_at: result_device.updated_at,
        deleted_at: result_device.deleted_at,
        message: DeviceMessageCreateResponse{
            uuid: result_message.uuid,
            device_uuid: result_device.uuid,
            topic: result_message.topic,
            qos: result_message.qos,
            retained: result_message.retained,
            publisher: result_message.publisher,
            subscriber: result_message.subscriber,
            command_start: result_message.command_start,
            command_end: result_message.command_end,
            command_last: result_message.command_last,
            command_last_time: result_message.command_last_time,
            created_at: result_message.created_at,
            updated_at: result_message.updated_at,
            deleted_at: result_message.deleted_at,
        },
        scale: result_scale.iter().map(|scale| DeviceScaleCreateResponse {
            uuid: scale.uuid,
            device_id: scale.device_id,
            metric: scale.metric.clone(),
            unit: scale.unit.clone(),
            created_at: scale.created_at,
            updated_at: scale.updated_at,
            deleted_at: scale.deleted_at,
        }).collect()
    };

    let _ = create_device_collection(
        app_state,
        &result.uuid,
        &result.user_uuid,
    ).await?;

    Ok(HttpResponse::Ok().json(&result))

}