use actix_web::web;
use crate::health_check::health_check_handler::health_check;

pub fn health_check_cfg(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
}