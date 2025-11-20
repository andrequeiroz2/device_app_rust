use actix_web::{web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use log::{error, info};
use mongodb::bson::{DateTime as BsonDateTime};
use mongodb::Client;
use uuid::Uuid;
use crate::auth::auth_tool::token_info;
use crate::broker::broker_tool::decode_received_message;
use crate::data_store::data_store_device_model::{DeviceData, DeviceDataStoreResponse};
use crate::data_store::data_store_device_query::{get_device_with_uuid_data_store_query, post_device_data_store_query, update_device_messages_query};
use crate::data_store::data_store_tool::bson_to_chrono;
use crate::error_app::error_app::AppError;
use crate::state::AppState;
use crate::user::user_query::get_user_by_uuid;
use crate::device::device_adoption_tool::device_decompose_topic;

pub async fn create_device_collection(
    app_state: web::Data<AppState>,
    device_uuid: &Uuid,
    user_uuid: &Uuid,
) -> Result<(), AppError> {

    info!("file: {}, lime: {}, device_uuid: {}, user_uuid: {}",
        file!(),
        line!(),
        device_uuid,
        user_uuid
    );
    
    let device = DeviceData {
        id: device_uuid.to_string(),
        device_uuid: device_uuid.to_string(),
        user_uuid: user_uuid.to_string(),
        messages: vec![],
        created_at: BsonDateTime::now(),
        updated_at: None,
        deleted_at: None,
    };

    post_device_data_store_query(&app_state.mongo, device).await?;

    Ok(())
}

pub async fn get_device_collection(
    app_state: web::Data<AppState>,
    device_uuid: web::Path<Uuid>,
    credentials: BearerAuth,
) -> Result<HttpResponse, AppError> {

    let token = token_info(credentials.token().to_string()).await?;
    let user = get_user_by_uuid(&app_state.db, &token.inf.uuid).await?;

    let device_data = get_device_with_uuid_data_store_query(
        &app_state.mongo,
        &device_uuid,
        &user.uuid
    ).await?;

    let updated_at;
    let deleted_at;

    let created_at = bson_to_chrono(&device_data.created_at)?;

    if let Some(i) = device_data.updated_at {
        updated_at = Some(bson_to_chrono(&i)?);
    }else{
        updated_at = None;
    };

    if let Some(i) = device_data.deleted_at {
        deleted_at = Some(bson_to_chrono(&i)?);
    }else {
        deleted_at = None;
    };

    let result = DeviceDataStoreResponse {
        id: Uuid::parse_str(&device_data.id).unwrap(),
        device_uuid: Uuid::parse_str(&device_data.device_uuid).unwrap(),
        user_uuid: Uuid::parse_str(&device_data.user_uuid).unwrap(),
        created_at,
        updated_at,
        deleted_at,
    };

    Ok(HttpResponse::Ok().json(result))

}

pub async fn put_device_collection(
    client: Client,
    message: &paho_mqtt::Message
) {
    let decode_message = match decode_received_message(message){
        Ok(decode) => decode,
        Err(err) => {
            error!("file: {}, line: {}, Failed to decode message: {:?}", file!(), line!(), err);
            return;
        }
    };

    let decompose_topic = match device_decompose_topic(&decode_message.topic){
        Ok(decompose) => decompose,
        Err(err) => {
            error!("file: {}, line: {}, Failed to decode message: {:?}", file!(), line!(), err);
            return;
        }
    };

    match update_device_messages_query(client, &decode_message, &decompose_topic).await{
        Ok(data) => data,
        Err(err) => {
            error!("file: {}, line: {}, Failed to update device messages: {:?}", file!(), line!(), err);
            return;
        }
    };

}