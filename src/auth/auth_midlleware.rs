use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{HttpMessage, Error};
use actix_web::middleware::Next;
use crate::error_app::error_app::{AppError, AppMsgError};
use jwt_lib::jwt_decode;
use crate::auth::auth_config::AuthConfig;

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>
) -> Result<ServiceResponse<impl MessageBody>,Error> {
    let auth = req.headers().get("Authorization");

    if auth.is_none() {
        Err(
            AppError::Unauthorized(
                AppMsgError{
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: "Authorization is empty".to_string()
                }
            )
        )?
    }

    let token = auth.unwrap().to_str().unwrap().replace("Bearer ", "").to_owned();
    
    let algorithm = AuthConfig::get_algorithm();

    let claim = match jwt_decode(algorithm, token.clone()){
        Ok(claim)=> claim,
        Err(err) => Err(
            AppError::Unauthorized(
                AppMsgError{
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: format!("{}, algorithm: {}, token: {}", err, algorithm, token)
                }
            )
        )?
    };

    req.extensions_mut().insert(claim);

    next.call(req).await
        .map_err(|err|
            Error::from(
                AppError::Unauthorized(
                    AppMsgError{
                        api_msg_error: "Unauthorized".to_string(),
                        log_msg_error: err.to_string()
                    }
                )
            )
        )
}