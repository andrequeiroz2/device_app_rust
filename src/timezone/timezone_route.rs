use actix_web::web;
use crate::timezone::timezone_handler::timezone_get;

pub fn timezone_cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/timezone")
        .route("", web::get().to(timezone_get))
    );
}