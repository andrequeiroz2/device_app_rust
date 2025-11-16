use log::info;
use mongodb::{Client, Collection};
use mongodb::bson::doc;
use mongodb::options::{FindOneOptions, UpdateOptions};
use mongodb::bson::{DateTime as BsonDateTime};
use uuid::Uuid;
use crate::data_store::data_store_device_model::DeviceData;
use crate::device::device_message_model::MessageReceivePayload;
use crate::error_app::error_app::{AppError, AppMsgError, AppMsgInfError};

pub async fn post_device_data_store_query(
    client: &Client,
    device: DeviceData,
) -> Result<(), AppError>{

    let database = client.database("devices");
    let collection: Collection<DeviceData> = database.collection("devices");

    collection.insert_one(device).await.map_err(|e| AppError::MongoDBError(AppMsgInfError {
        file: file!().to_string(),
        line: line!(),
        api_msg_error: "Internal server error".into(),
        log_msg_error: e.to_string(),
    }))?;

    Ok(())
}

pub async fn get_device_with_uuid_data_store_query(
    client: &Client,
    device_uuid: &Uuid,
    user_uuid: &Uuid,
) -> Result<DeviceData, AppError>{

    info!("file: {}, line: {}, device_uuid: {}, user_uuid: {:?}",
        file!(),
        line!(),
        device_uuid,
        user_uuid
    );

    let database = client.database("devices");
    let collection: Collection<DeviceData> = database.collection("devices");

    let filter = doc! {
        "_id": device_uuid.to_string(),
        "user_uuid": user_uuid.to_string(),
    };

    let field_response = FindOneOptions::builder()
        .projection(doc! { "messages": 0 })
        .build();

    match collection.find_one(filter).with_options(field_response).await {
        Ok(Some(device)) => Ok(device),

        Ok(None) => Err(AppError::NotFound(
            AppMsgError{
                api_msg_error: "Device not found".into(),
                log_msg_error: format!("file: {}, line: {}, Device not found: device_uuid: {}",
                    file!(),
                    line!(),
                    device_uuid.to_string()
                )
            }))?,

        Err(e) => Err(AppError::MongoDBError(
            AppMsgInfError{
                file: file!().to_string(),
                line: line!(),
                api_msg_error: "Internal server error".into(),
                log_msg_error: e.to_string(),
            }))?,
    }
}


pub async fn update_device_messages_query(
    client: Client,
    message: &MessageReceivePayload,
) -> Result<(), AppError> {
    info!(
        "file: {}, line: {}, message: {:?}",
        file!(),
        line!(),
        message
    );

    let database = client.database("devices");
    let collection: Collection<mongodb::bson::Document> = database.collection("devices");

    let filter = doc! {
        "_id": message.device_uuid.to_string(),
        "user_uuid": message.user_uuid.to_string(),
    };

    let update = doc! {
        "$push": {
            "messages": {
                "value": message.payload.clone(),
                "metric": message.metric.clone(),
                "scale": message.scale.clone(),
                "timestamp": BsonDateTime::now(),
            }
        },
        "$set": {
            "updated_at": BsonDateTime::now()
        }
    };

    let update_options = UpdateOptions::builder().upsert(false).build();

    match collection.update_one(filter, update).await {
        Ok(result) if result.matched_count > 0 => Ok(()),

        Ok(_) => Err(AppError::NotFound(AppMsgError {
            api_msg_error: "Device not found".into(),
            log_msg_error: format!(
                "file: {}, line: {}, Device not found: device_uuid: {}, user_uuid: {}",
                file!(),
                line!(),
                message.device_uuid.to_string(),
                message.user_uuid.to_string()
            ),
        }))?,

        Err(e) => Err(AppError::MongoDBError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "Internal server error".into(),
            log_msg_error: e.to_string(),
        }))?,
    }
}