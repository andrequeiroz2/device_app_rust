use log::{error, info};
use mongodb::{Client, Collection};
use mongodb::bson::{doc, from_document, to_document, Bson};
use mongodb::options::FindOneOptions;
use mongodb::bson::{DateTime as BsonDateTime};
use uuid::Uuid;
use crate::data_store::data_store_device_model::{DeviceData, DeviceMessageReceived, DeviceMessagesOwned};
use crate::device::device_adoption_tool::DecomposeTopic;
use crate::device::device_message_model::MessageReceivePayload;
use crate::error_app::error_app::{AppError, AppMsgError, AppMsgInfError};
use chrono::DateTime;
use futures_util::TryStreamExt;

pub async fn post_device_data_store_query(
    client: &Client,
    device: DeviceData,
) -> Result<(), AppError>{

    let database = client.database("devices");
    let collection: Collection<mongodb::bson::Document> = database.collection("devices");

    // Converter DeviceData para Document, mas garantir que messages seja objeto {} em vez de array []
    let mut doc = match to_document(&device) {
        Ok(doc) => doc,
        Err(e) => {
            return Err(AppError::MongoDBError(AppMsgInfError {
                file: file!().to_string(),
                line: line!(),
                api_msg_error: "Internal server error".into(),
                log_msg_error: format!("Error converting DeviceData to Document: {}", e),
            }));
        }
    };

    // Garantir que messages seja sempre um objeto vazio {}, não array []
    doc.insert("messages", mongodb::bson::Bson::Document(mongodb::bson::Document::new()));

    collection.insert_one(doc).await.map_err(|e| AppError::MongoDBError(AppMsgInfError {
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
    decompose_topic: &DecomposeTopic
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
        "_id": decompose_topic.device_uuid.to_string(),
        "user_uuid": decompose_topic.user_uuid.to_string(),
    };

    let dt = match DateTime::parse_from_rfc3339(&message.timestamp){
        Ok(dt) => dt,
        Err(error) => {
            error!("file: {}, line: {}, error: {}, timestamp: {}", file!(), line!(), error, &message.timestamp);
            Err(AppError::BadRequest(format!("file: {}, line:{}, Invalid timestamp, timestamp: {}", file!(), line!(), &message.timestamp)))?
        }
    };

    let message_received = DeviceMessageReceived {
        value: message.payload.clone(),
        scale: message.scale.clone(),
        timestamp: dt,
    };

    let message_bson = match mongodb::bson::to_bson(&message_received) {
        Ok(bson) => bson,
        Err(error) => {
            Err(AppError::MongoDBError(AppMsgInfError {
                file: file!().into(),
                line: line!(),
                api_msg_error: "Internal server error".into(),
                log_msg_error: format!("file: {}, line: {}, error: {}", file!(), line!(), error)
            }))?
        }
    };

    let push_path = format!("messages.{}", message.metric);

    let _ = collection.update_one(
        doc! {
            "_id": decompose_topic.device_uuid.to_string(),
            "user_uuid": decompose_topic.user_uuid.to_string(),
            "$or": [
                { "messages": { "$exists": false } },
                { "messages": { "$type": "array" } }
            ]
        },
        doc! {
            "$set": {
                "messages": {}
            }
        }
    ).await;

    let update = doc! {
        "$push": {
            &push_path: message_bson
        },
        "$set": {
            "updated_at": BsonDateTime::now()
        }
    };

    let result = collection.update_one(filter, update).await.map_err(|e| {
        AppError::MongoDBError(AppMsgInfError {
            file: file!().into(),
            line: line!(),
            api_msg_error: "Internal server error".into(),
            log_msg_error: e.to_string(),
        })
    })?;

    if result.matched_count == 0 {
        Err(AppError::NotFound(AppMsgError {
            api_msg_error: "Device not found".into(),
            log_msg_error: format!(
                "file: {}, line: {}, Device not found: device_uuid: {}, user_uuid: {}",
                file!(),
                line!(),
                decompose_topic.device_uuid,
                decompose_topic.user_uuid,
            ),
        }))?;
    }

    Ok(())
}

pub async fn get_message_data_store_query(
    client: &Client,
    device_uuids: Vec<Uuid>,
)-> Result<Vec<DeviceMessagesOwned>, AppError>{

    let database = client.database("devices");

    let devices_uuid: Vec<String> = device_uuids
        .iter()
        .map(|u| u.to_string())
        .collect();

    let pipeline = vec![
        //match - user device_uuid or _id fallback
        doc! {
            "$match": {
                "$or": [
                    { "device_uuid": { "$in": &devices_uuid } },
                    { "_id": { "$in": &devices_uuid } }
                ],
                "deleted_at": Bson::Null
            }
        },

        //use device_uuid if exists, or _id
        doc! {
            "$project": {
                "device_uuid": {
                    "$ifNull": ["$device_uuid", "$_id"]
                },
                "messages": {
                    "$ifNull": ["$messages", {}]
                }
            }
        },

        doc! {
            "$addFields": {
                "messages": {
                    "$cond": {
                        "if": { "$isArray": "$messages" },
                        "then": {},  // Se for array, usar objeto vazio
                        "else": "$messages"  // Se for objeto, usar diretamente
                    }
                }
            }
        },

        doc! {
            "$project": {
                "device_uuid": 1,
                "metrics": { "$objectToArray": "$messages" }
            }
        },

        // Unwind para ter uma linha por métrica
        doc! {
            "$unwind": "$metrics"
        },

        // Unwind o array de mensagens de cada métrica
        doc! {
            "$unwind": "$metrics.v"
        },

        //sort for timestamp desc
        doc! {
            "$sort": {
                "metrics.v.timestamp": -1
            }
        },

        //group for a device + type - pegar apenas o último valor (mais recente)
        doc! {
            "$group": {
                "_id": {
                    "device_uuid": "$device_uuid",
                    "type": "$metrics.k"
                },
                "last_value": { "$first": "$metrics.v" }
            }
        },

        doc! {
            "$match": {
                "_id.type": { "$ne": Bson::Null },
                "last_value": { "$ne": Bson::Null }
            }
        },

        doc! {
            "$group": {
                "_id": "$_id.device_uuid",
                "messages": {
                    "$push": {
                        "k": "$_id.type",
                        "v": {
                            "value": "$last_value.value",
                            "scale": "$last_value.scale",
                            "timestamp": "$last_value.timestamp"
                        }
                    }
                }
            }
        },
        //project final
        doc! {
            "$project": {
                "_id": 0,
                "device_uuid": "$_id",
                "messages": { "$arrayToObject": "$messages" }
            }
        },
    ];

    let collection: Collection<mongodb::bson::Document> = database.collection("devices");

    let cursor = match collection
        .aggregate(pipeline)
        .await{
            Ok(cursor) => cursor,
            Err(e) => Err(AppError::MongoDBError(
                AppMsgInfError{
                    file: file!().to_string(),
                    line: line!(),
                    api_msg_error: "Internal server error".into(),
                    log_msg_error: e.to_string(),
                }))?,
        };

    let results: Vec<mongodb::bson::Document> =
        match cursor.try_collect().await{
            Ok(docs) => docs,
            Err(e) => Err(AppError::MongoDBError(
                AppMsgInfError{
                    file: file!().to_string(),
                    line: line!(),
                    api_msg_error: "Internal server error".into(),
                    log_msg_error: e.to_string(),
                }))?,
        };


    let mut device_messages = Vec::with_capacity(results.len());
    device_messages.extend(
        results
            .into_iter()
            .filter_map(|doc| {
                match from_document::<DeviceMessagesOwned>(doc) {
                    Ok(device_msg) => Some(device_msg),
                    Err(e) => {
                        error!("file: {}, line: {}, Error convert document: {}", file!(), line!(), e);
                        None
                    }
                }
            }
        )
    );

    Ok(device_messages)
}