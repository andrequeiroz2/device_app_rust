use sqlx::postgres::PgPool;
use actix_web::web::Data;


pub struct AppState{
    pub health_check: String,
    pub db: PgPool
}

pub fn app_state(db_pool: PgPool) -> Data<AppState>{

    let shared_data= Data::new(AppState {
        health_check: "I'm good.".to_string(),
        db: db_pool,
    });

    shared_data
}