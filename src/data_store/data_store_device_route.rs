use actix_web::middleware::from_fn;
use actix_web::web;
use crate::auth;
use crate::data_store::data_store_device_handler::get_device_collection;

pub fn data_store_device_cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/device_data_store")
            .wrap(from_fn(auth::auth_midlleware::auth_middleware))
            .route("/{uuid}", web::get().to(get_device_collection))
    );
}