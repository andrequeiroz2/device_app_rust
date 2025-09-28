use sqlx::postgres::PgPool;
use actix_web::web::Data;
use crate::broker::broker_model::BrokerManager;

pub struct AppState{
    pub health_check: String,
    pub db: PgPool,
    pub broker_manager: BrokerManager,
}

pub fn app_state(db_pool: PgPool) -> Data<AppState>{

    let shared_data= Data::new(AppState {
        health_check: "I'm good.".to_string(),
        db: db_pool,
        broker_manager: BrokerManager::default(),
    });

    shared_data
}