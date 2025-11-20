use actix_web::{web, HttpResponse};
use actix_web::web::Json;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use log::info;
use crate::auth::auth_tool::token_info;
use crate::device::device_adoption_model::DeviceAdoptionRequest;
use crate::device::device_model::DeviceFilter;
use crate::device::device_query::get_device_filter;
use crate::error_app::error_app::AppError;
use crate::state::AppState;
use crate::user::user_query::get_user_by_uuid;

pub async fn device_adoption(
    device_adoption_request: Json<DeviceAdoptionRequest>,
    credentials: BearerAuth,
    app_state: web::Data<AppState>,
)-> Result<HttpResponse, AppError>{

    let token = token_info(credentials.token().to_string()).await?;

    let user = get_user_by_uuid(&app_state.db, &token.inf.uuid).await?;

    let device_adoption = device_adoption_request.into_inner();

    let _ = match device_adoption.validate(){
        Ok(result) => result,
        Err(err) => Err(err)?
    };

    let device_filter = DeviceFilter{
        uuid: None,
        mac_address: Some(device_adoption.mac_address.clone()),
    };

    let device_query_result = get_device_filter(&app_state.db, &device_filter).await?;

    if let Some(device_query_result) = device_query_result {
        info!("Device already registered: {}", device_query_result.mac_address);
        Err(AppError::BadRequest("Device already registered".to_string()))?
    } else {
        info!("Device not registered: {}", device_adoption.mac_address);
    }

    Ok(HttpResponse::Ok().json(""))
}