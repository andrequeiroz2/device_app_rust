use actix_web::{web, HttpResponse};
use uuid::Uuid;
use web::Json;
use crate::broker::broker_model::{BrokerCreate, BrokerFilter, BrokerUpdate};
use crate::broker::broker_query::{
    delete_broker_query, 
    get_broker_query, 
    get_broker_update_check_query, 
    get_broker_with_uuid_query, 
    post_broker_query, 
    put_broker_query
};
use crate::error_app::error_app::AppError;
use crate::state::AppState;

pub async fn broker_create(
    broker: Json<BrokerCreate>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{

    post_broker_query(&app_state.db, broker.into(), &Uuid::new_v4())
        .await
        .map(|broker| HttpResponse::Ok().json(broker))
}

pub async fn broker_get_filter(
    filter: web::Query<BrokerFilter>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{

    get_broker_query(&app_state.db, &filter)
        .await
        .map(|broker| HttpResponse::Ok().json(broker))
}

pub async fn broker_soft_delete(
    broker_uuid: web::Path<Uuid>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{

    let broker = match get_broker_with_uuid_query(&app_state.db, &broker_uuid).await{
        Ok(broker) => broker,
        Err(e) => Err(e)?
    };

    delete_broker_query(&app_state.db, &broker.uuid)
        .await
        .map(|_| HttpResponse::NoContent().finish())
}


pub async fn broker_update(
    broker_uuid: web::Path<Uuid>,
    broker_update: Json<BrokerUpdate>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{
    
    let broker_uuid = broker_uuid.into_inner();

    let broker = match get_broker_with_uuid_query(&app_state.db, &broker_uuid).await{
        Ok(broker) => broker,
        Err(e) => Err(e)?
    };

    match get_broker_update_check_query(&app_state.db, &broker_uuid, &broker_update)
        .await{
        Ok(_) => (),
        Err(err) => Err(err)?
    }

    put_broker_query(&app_state.db, &broker_uuid, &broker_update)
        .await
        .map(|broker| HttpResponse::Ok().json(broker))
}