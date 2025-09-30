use sqlx::postgres::PgPool;
use actix_web::web::Data;
use mongodb::Client;
use crate::broker::broker_model::BrokerManager;
use crate::database::connection_mongo::get_mongo_client;


pub struct AppState{
    pub health_check: String,
    pub mongo: Client,
    pub db: PgPool,
    pub broker_manager: BrokerManager,
}

pub async fn app_state(db_pool: PgPool) -> Data<AppState>{

    let mongo_client = get_mongo_client().await;

    let shared_data= Data::new(AppState {
        health_check: "I'm good.".to_string(),
        db: db_pool,
        mongo: mongo_client,
        broker_manager: BrokerManager::default(),
    });

    shared_data
}