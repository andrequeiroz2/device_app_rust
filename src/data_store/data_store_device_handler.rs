use actix_web::web;
use mongodb::bson::{DateTime as BsonDateTime};
use mongodb::{
    bson::doc,
    options::{IndexOptions},
    Collection, IndexModel,
};
use uuid::Uuid;
use crate::data_store::data_store_device_model::DeviceData;
use crate::error_app::error_app::{AppError, AppMsgInfError};
use crate::state::AppState;

pub async fn create_device_collection(
    app_state: web::Data<AppState>,
    device_uuid: &Uuid,
    user_uuid: &Uuid,
) -> Result<(), AppError> {
    // Usa o nome do banco definido na configuração
    let database = app_state.mongo.database("devices_db"); // substitua pelo nome do seu DB
    let coll: Collection<DeviceData> = database.collection("devices");

    // Upsert: cria o documento apenas se não existir
    let filter = doc! { "device_uuid": device_uuid.to_string() };
    let update = doc! {
        "$setOnInsert": {
            "device_uuid": device_uuid.to_string(),
            "user_uuid": user_uuid.to_string(),
            "messages": [],
            "created_at": BsonDateTime::now(),
        }
    };

    coll.update_one(filter, update)
        .await
        .map_err(|e| AppError::MongoDBError(AppMsgInfError{
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "Internal server error".into(),
            log_msg_error: e.to_string(),
        }))?;

    Ok(())
}