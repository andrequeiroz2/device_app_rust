use actix_web::web;
use crate::device::device_handler::device_create;

pub fn device_cfg(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/device")
            .route("", web::post().to(device_create))
    );
}