use actix_web::web;
use crate::auth::auth_handler::auth_login;


pub fn auth_cfg(cfg: &mut web::ServiceConfig){
    cfg.service(web::scope("/login")
        .route("", web::post().to(auth_login))
    );
}