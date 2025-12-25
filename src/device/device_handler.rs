use actix_web::{web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use uuid::Uuid;
use web::Json;
use crate::auth::auth_tool::token_info;
use crate::broker::broker_model::BrokerManager;
use crate::broker::broker_query::{get_broker_connected_query};
use crate::broker::broker_tool::build_subscribe_topic_qos;
use crate::data_store::data_store_device_handler::create_device_collection;
use crate::data_store::data_store_device_query::get_message_data_store_query;
use crate::device::device_adoption_tool::device_compose_topic;
use crate::device::device_model::{DeviceAndMessageResponse, DeviceCreate, DeviceCreateRequest, DeviceCreateResponse, DeviceFilter, DevicePaginationFilter, DevicePaginationResponse};
use crate::device::device_query::{get_device_filter, post_device_message_query, get_devices_owned_by_user, get_device_count_total_owned_user};
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::state::AppState;
use crate::user::user_query::get_user_by_uuid;
use crate::device::device_message_model::{DeviceMessageCreateResponse, DeviceScaleCreateResponse};
use std::collections::HashMap;
use crate::data_store::data_store_device_model::DeviceMessagesOwned;

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

    let broker = match get_broker_connected_query(&app_state.db).await{
        Ok(broker)=> {
            match broker { 
                Some(broker) => broker,
                None => Err(
                    AppError::NotFound(
                        AppMsgError {
                            api_msg_error: "Broker not connected. Connect to an MQTT broker before registering a device".into(),
                            log_msg_error: "Broker not connected. Connect to an MQTT broker before registering a device".into(),
                        }
                    )
                )?
            }
        } 
        Err(err) => Err(
            AppError::NotFound(
                AppMsgError {
                    api_msg_error: format!("Broker not connected. Connect to an MQTT broker before registering a device, error: {}", err),
                    log_msg_error: "Broker not connected. Connect to an MQTT broker before registering a device".into(),
                }
            )
        )?
    };
    
    let topic_compose = device_compose_topic(&user.uuid, &device.uuid, &device.name);

    mqtt_device::components::topic::valid_topic(&topic_compose)
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let (result_device, result_message, result_scale) = post_device_message_query(&app_state.db, &device, topic_compose.clone()).await?;
    
    if device.device_type_int == 0 {
        let _ = build_subscribe_topic_qos(broker.uuid, topic_compose, device.message.qos,  manager.clone()).await?;
    }
    
    let broker_url = format!("{}:{}", broker.host, broker.port);
    
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
        }).collect(),
        broker_url
    };

    let _ = create_device_collection(
        app_state,
        &result.uuid,
        &result.user_uuid,
    ).await?;

    Ok(HttpResponse::Ok().json(&result))

}

pub async fn devices_owned_by_user(
    credentials: BearerAuth,
    app_state: web::Data<AppState>,
    pagination: web::Query<DevicePaginationFilter>,
)-> Result<HttpResponse, AppError>{

    let token = token_info(credentials.token().to_string()).await?;
    let user = get_user_by_uuid(&app_state.db, &token.inf.uuid).await?;

    let devices = get_devices_owned_by_user(&app_state.db, user.id, &pagination).await?;

    let devices_count;
    if devices.len() == 0 {
        devices_count = 0;
    }else {
        devices_count = get_device_count_total_owned_user(&app_state.db, user.id).await?;
    }

    let device_uuids: Vec<Uuid> = devices.iter().map(|d| d.uuid.clone()).collect();

    let messages = get_message_data_store_query(&app_state.mongo, device_uuids).await?;

    let messages_map: HashMap<String, DeviceMessagesOwned> = messages
        .into_iter()
        .map(|msg| {
        
            let normalized_uuid = match Uuid::parse_str(&msg.device_uuid) {
                Ok(uuid) => uuid.to_string(),
                Err(_) => msg.device_uuid.clone(),
            };
        
            (normalized_uuid, msg)
        })
    .collect();

    let devices_with_messages: Vec<DeviceAndMessageResponse> = devices
        .iter()
        .map(|device| {
            let device_uuid_str = device.uuid.to_string();
            let messages = messages_map.get(&device_uuid_str).map(|msg| {
                vec![DeviceMessagesOwned {
                    device_uuid: msg.device_uuid.clone(),
                    messages: msg.messages.clone(),
                }]
            });
            DeviceAndMessageResponse {
                uuid: device.uuid,
                user_id: device.user_id,
                name: device.name.clone(),
                device_type_int: device.device_type_int,
                device_type_text: device.device_type_text.clone(),
                board_type_int: device.board_type_int,
                board_type_text: device.board_type_text.clone(),
                sensor_type: device.sensor_type.clone(),
                actuator_type: device.actuator_type.clone(),
                device_condition_int: device.device_condition_int,
                device_condition_text: device.device_condition_text.clone(),
                mac_address: device.mac_address.clone(),
                message: messages,
                created_at: device.created_at,
                updated_at: device.updated_at,
                deleted_at: device.deleted_at,
            }
        })
    .collect();

    // Converter paginação de String para u32
    let page: String;
    let page_size: String;

    if pagination.pagination.page.is_empty() {
        page = "1".to_string();
    } else {
        page = pagination.pagination.page.clone();
    }

    if pagination.pagination.page_size.is_empty() {
        page_size = "10".to_string();
    } else {
        page_size = pagination.pagination.page_size.clone();
    }

    let pagination_from = match crate::paginate::paginate_model::Pagination::new(page, page_size) {
        Ok(result) => result,
        Err(err) => Err(err)?,
    };

    let result = DevicePaginationResponse::new(
        devices_with_messages,
        devices_count,
        pagination_from.page,
        pagination_from.page_size,
    );

    Ok(HttpResponse::Ok().json(&result))
}