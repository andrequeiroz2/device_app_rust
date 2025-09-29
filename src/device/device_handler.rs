use actix_web::{web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use web::Json;
use crate::auth::auth_tool::token_info;
use crate::broker::broker_model::{BrokerFilter, BrokerManager};
use crate::broker::broker_query::{get_broker_connected_query, get_broker_query, get_broker_with_uuid_query};
use crate::broker::broker_tool::build_subscribe_topic_qos;
use crate::device::device_model::{DeviceCreate, DeviceCreateRequest, DeviceCreateResponse, DeviceFilter};
use crate::device::device_query::{get_device_filter, post_device_message_query};
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::state::AppState;
use crate::user::user_query::get_user_by_uuid;
use crate::device::device_message_model::DeviceMessageCreateResponse;

pub async fn device_create(
    device: Json<DeviceCreateRequest>,
    credentials: BearerAuth,
    app_state: web::Data<AppState>,
    manager: web::Data<BrokerManager>
)-> Result<HttpResponse, AppError>{

    let token = token_info(credentials.token().to_string()).await?;

    let user = get_user_by_uuid(&app_state.db, &token.inf.uuid).await?;

    let device = device.into_inner();
    let device = DeviceCreate::new(device, user.id).await?;

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

    let (result_device, result_message) = post_device_message_query(&app_state.db, device.clone()).await?;

    let broker = get_broker_connected_query(&app_state.db).await?;

    if broker.is_some(){
        let broker = broker.unwrap();
        let _ = build_subscribe_topic_qos(broker.uuid, device.message.topic, device.message.qos,  manager.clone()).await?;
    }

    let result = DeviceCreateResponse{
        uuid: result_device.uuid,
        user_uuid: user.uuid,
        name: result_device.name,
        device_type_int: result_device.device_type_int,
        device_type_text: result_device.device_type_text,
        border_type_int: result_device.border_type_int,
        border_type_text: result_device.border_type_text,
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
            payload: result_message.payload,
            qos: result_message.qos,
            retained: result_message.retained,
            publisher: result_message.publisher,
            subscriber: result_message.subscriber,
            scale: result_message.scale,
            command_start: result_message.command_start,
            command_end: result_message.command_end,
            command_last: result_message.command_last,
            command_last_time: result_message.command_last_time,
            created_at: result_message.created_at,
            updated_at: result_message.updated_at,
            deleted_at: result_message.deleted_at,
        }
    };

    Ok(HttpResponse::Ok().json(result))

}