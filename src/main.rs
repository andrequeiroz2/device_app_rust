mod database;
mod state;
mod health_check;
mod error_app;
mod user;
mod auth;
mod broker;
pub mod device;
pub mod paginate;
mod timezone;
mod data_store;

use std::io;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use health_check::health_check_cfg::health_check_cfg;
use auth::auth_route::auth_cfg;
use user::user_route::user_cfg;
use auth::auth_config::AuthConfig;
use crate::broker::broker_model::BrokerManager;
use crate::broker::broker_route::broker_cfg;
use crate::data_store::data_store_device_route::data_store_device_cfg;
use crate::database::connection_mongo::init_devices_collection;
use crate::device::device_route::device_cfg;
use crate::state::app_state;
use crate::timezone::timezone_route::timezone_cfg;

#[actix_web::main]
async fn main()-> io::Result<()> {

    dotenv().ok();
    log4rs::init_file("log_config.yml", Default::default()).expect("Log config file not found.");

    //Init auth config and set auth keys path
    AuthConfig::init_auth_config();
    AuthConfig::set_auth_keys(
        AuthConfig::get_private_key_path(),
        AuthConfig::get_public_key_path()
    ).await;

    let dp_postgres_pool = database::connection_postgres::get_postgres_pool().await;
    let shared_data = state::app_state(dp_postgres_pool).await;

    let _= init_devices_collection(
        shared_data.clone(),
        "devices").await.expect("Failed to initialize devices collection");


    let broker_manager = BrokerManager::default();
    
    let app = move ||{
        App::new()
            .app_data(shared_data.clone())
            .app_data(web::Data::new(broker_manager.clone()))
            .configure(health_check_cfg)
            .configure(auth_cfg)
            .configure(user_cfg)
            .configure(timezone_cfg)
            .configure(broker_cfg)
            .configure(device_cfg)
            .configure(data_store_device_cfg)
    };


    let host_address = std::env::var("HOST_ADDRESS")
    .expect("HOST_ADDRESS must be specified");

    let server_address = host_address.as_str();

    HttpServer::new(app).bind(server_address)?.run().await
}
