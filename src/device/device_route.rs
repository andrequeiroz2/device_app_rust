use actix_web::middleware::from_fn;
use actix_web::web;
use crate::auth;
use crate::device::device_handler::{device_create, devices_owned_by_user};

pub fn device_cfg(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/device")
            .wrap(from_fn(auth::auth_midlleware::auth_middleware))
            .route("", web::post().to(device_create))
            .route("/owned", web::get().to(devices_owned_by_user))
    );
}