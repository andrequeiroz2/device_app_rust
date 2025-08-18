use actix_web::{web, HttpResponse};
use web::Json;
use crate::device::device_model::DeviceCreate;
use crate::device::device_query::post_device_query;
use crate::error_app::error_app::AppError;
use crate::state::AppState;

pub async fn device_create(
    device: Json<DeviceCreate>,
    app_state: web::Data<AppState>
)-> Result<HttpResponse, AppError>{

    post_device_query(&app_state.db, device.into()).await.map(
        |device| HttpResponse::Ok().json(device)
    )
}