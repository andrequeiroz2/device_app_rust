use std::sync::Arc;
use actix_web::{web, HttpResponse};
use log::info;
use uuid::Uuid;
use web::Json;
use crate::broker::broker_model::{BrokerCreate, BrokerFilter, BrokerManager, BrokerUpdate};
use crate::broker::broker_query::{delete_broker_query, get_broker_count_query, get_broker_query, get_broker_update_check_query, get_broker_with_uuid_query, post_broker_query, put_broker_query, put_broker_state_query};
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::state::AppState;
use crate::broker::broker_connection as mod_broker_connection;
use crate::broker::broker_tool::broker_change_state;

pub async fn broker_create(
    broker: Json<BrokerCreate>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{

    let broker = broker.into_inner();

    let broker_check = get_broker_count_query(&app_state.db, broker.port)
        .await
        .map_err(|e| e)?;

    if broker_check.is_some(){
        return Err(AppError::ConstraintViolation(
            AppMsgError{
                api_msg_error: "Broker port already registered".to_string(),
                log_msg_error: format!("Broker port already registered, port: {}", broker.port)
            }
        ))?
    }

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

pub async fn broker_delete(
    broker_uuid: web::Path<Uuid>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{

    let broker = get_broker_with_uuid_query(&app_state.db, &broker_uuid).await?;
    let broker = broker.unwrap();

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

    let broker_port = match get_broker_update_check_query(&app_state.db, &broker_uuid, &broker_update)
        .await{
        Ok(result) => result,
        Err(err) => Err(err)?
    };

    if broker_port > 0{
        return Err(AppError::ConstraintViolation(
            AppMsgError{
                api_msg_error: "Broker port already registered".to_string(),
                log_msg_error: format!("Broker port already registered, port: {}", broker_update.port)
            }
        ))?
    }

    match get_broker_with_uuid_query(&app_state.db, &broker_uuid).await{
        Ok(broker) => broker,
        Err(e) => Err(e)?
    };

    put_broker_query(&app_state.db, &broker_uuid, &broker_update)
        .await
        .map(|broker| HttpResponse::Ok().json(broker))
}

pub async fn broker_connection(
    broker_uuid: web::Path<Uuid>,
    app_state: web::Data<AppState>,
    broker_manager: web::Data<BrokerManager>
) -> Result<HttpResponse, AppError>{

    let broker_uuid = broker_uuid.into_inner();

    let broker = get_broker_with_uuid_query(&app_state.db, &broker_uuid).await?;

    if broker.is_none(){
        return Err(AppError::NotFound(
            AppMsgError{
                api_msg_error: "Broker not found".to_string(),
                log_msg_error: format!("Broker not found, uuid: {}", broker_uuid)
            }
        ))?
    }

    let broker = broker.unwrap();

    if broker.connected{
        return Ok(HttpResponse::NoContent().finish())
    };

    mod_broker_connection::connect(&app_state.db, app_state.mongo.clone(), &broker, broker_manager.clone()).await?;

    broker_change_state(&broker.uuid, true, &app_state.db, false).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn broker_disconnect(
    broker_uuid: web::Path<Uuid>,
    manager: web::Data<BrokerManager>,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AppError>{
    
    let broker_uuid = broker_uuid.into_inner();

    if let Some(handle) = manager.get(&broker_uuid).await{

        info!("file: {}, line: {} Broker {} found, cancelling...",
            file!(),
            line!(),
            broker_uuid
        );

        handle.cancel_token.cancel();

        broker_change_state(&broker_uuid, false, &app_state.db, true).await?;

        Ok(HttpResponse::NoContent().finish())

    }else {

        Err(
            AppError::NotFound(
                AppMsgError{
                    api_msg_error: "Broker not found or not connected".to_string(),
                    log_msg_error: format!("Broker not found or not connected in BrokerManager, uuid: {}", broker_uuid)
                }
            )
        )?
    }
}