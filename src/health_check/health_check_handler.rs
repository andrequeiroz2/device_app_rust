use actix_web::{web, HttpResponse};
use crate::state::AppState;

pub async fn health_check(app_state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body("OK")
}