use actix_web::middleware::from_fn;
use actix_web::web;
use crate::auth;
use crate::broker::broker_handler::{broker_connection, broker_create, broker_get_filter, broker_delete, broker_update};

pub fn broker_cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/broker")
            .wrap(from_fn(auth::auth_midlleware::auth_middleware))
            .route("", web::post().to(broker_create))
            .route("", web::get().to(broker_get_filter))
            .route("/{uuid}", web::delete().to(broker_delete))
            .route("/{uuid}", web::put().to(broker_update))
            .route("/{uuid}", web::post().to(broker_connection))
    );
}