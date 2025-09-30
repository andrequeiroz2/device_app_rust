use actix_web::web;
use log::info;
use mongodb::{Client, Collection, IndexModel};
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use crate::data_store::data_store_device_model::DeviceData;
use crate::error_app::error_app::{AppError, AppMsgInfError};
use crate::state::AppState;

struct MongoConfig{
    database_url: String,
}

impl MongoConfig{
    fn init_mongo_config() -> MongoConfig{
        MongoConfig{
            database_url: std::env::var("DATABASE_URL_MONGO")
                .expect("DATABASE_URL_MONGO must be specified"),
        }
    }

    pub fn get_database_url(&self) -> &str{
        &self.database_url
    }
}

pub async fn get_mongo_client() -> mongodb::Client{
    let config_mongo = MongoConfig::init_mongo_config();

    let client = mongodb::Client::with_uri_str(&config_mongo.get_database_url())
        .await
        .expect("💥 Failed to connect to the target Mongo server!");

    info!("🍃 Successfully connected to target MongoDB server!");

    client
}

pub async fn init_devices_collection(app_state: web::Data<AppState>, db_name: &str) -> Result<(), AppError> {
    let db = app_state.mongo.database(db_name);
    let coll: Collection<DeviceData> = db.collection("devices");

    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "device_uuid": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_uuid": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "messages.timestamp": 1 })
            .build(),
    ];

    coll.create_indexes(indexes).await.map_err(|e| {
        AppError::MongoDBError(
            AppMsgInfError{
                file: file!().to_string(),
                line: line!(),
                api_msg_error: "Internal server error".into(),
                log_msg_error: e.to_string()
            })
    })?;

    Ok(())
}