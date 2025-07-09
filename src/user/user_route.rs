use actix_web::web;
use crate::user::user_handler::{
    user_create,
    user_get_filter,
    user_soft_delete,
    user_update
};

pub fn user_cfg(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/user")
            .route("/create", web::post().to(user_create))
            .route("", web::get().to(user_get_filter))
            .route("/{uuid}", web::delete().to(user_soft_delete))
            .route("/{uuid}", web::put().to(user_update))
    );
}