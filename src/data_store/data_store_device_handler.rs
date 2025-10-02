use actix_web::web;
use log::info;
use mongodb::bson::{DateTime as BsonDateTime};
use mongodb::Collection;
use uuid::Uuid;
use crate::data_store::data_store_device_model::DeviceData;
use crate::error_app::error_app::{AppError, AppMsgInfError};
use crate::state::AppState;

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

    let database = app_state.mongo.database("devices");
    let coll: Collection<DeviceData> = database.collection("devices");

    let device = DeviceData {
        id: device_uuid.to_string(),
        device_uuid: device_uuid.to_string(),
        user_uuid: user_uuid.to_string(),
        messages: vec![],
        created_at: BsonDateTime::now(),
        updated_at: None,
        deleted_at: None,
    };

    coll.insert_one(device).await.map_err(|e| AppError::MongoDBError(AppMsgInfError {
        file: file!().to_string(),
        line: line!(),
        api_msg_error: "Internal server error".into(),
        log_msg_error: e.to_string(),
    }))?;

    Ok(())
}