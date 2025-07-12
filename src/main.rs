mod database;
mod state;
mod health_check;
mod error_app;
mod user;
mod auth;

use std::io;
use actix_web::{App, HttpServer};
use dotenv::dotenv;
use health_check::health_check_cfg::health_check_cfg;
use auth::auth_route::auth_cfg;
use user::user_route::user_cfg;
use auth::auth_config::AuthConfig;
use crate::auth::auth_config;

#[actix_rt::main]
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
    let shared_data = state::app_state(dp_postgres_pool);
    
    let app = move ||{
        App::new()
            .app_data(shared_data.clone())
            .configure(health_check_cfg)
            .configure(auth_cfg)
            .configure(user_cfg)

    };

    let host_address = std::env::var("HOST_ADDRESS")
    .expect("HOST_ADDRESS must be specified");

    let server_address = host_address.as_str();

    HttpServer::new(app).bind(server_address)?.run().await
}
