use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_web::{web, HttpResponse};
use crate::auth::auth_model::LoginRequest;
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::state::AppState;
use crate::user::user_query::get_user_full_row;
use crate::auth::auth_tool::verify_password;
use jwt_lib::jwt_encode;
use jwt_lib::components::claims::JwtClaims;
use crate::AuthConfig;
use serde_json::json;

pub async fn auth_login(
    login: web::Json<LoginRequest>,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AppError>{

    let login = login.into_inner();

    let user = match get_user_full_row(&app_state.db, &login.email).await{
        Ok(user) => user,
        Err(err) => Err(err)?
    };

    match verify_password(&login.password, &user.password) {
        Ok(result) => result,
        Err(err) => Err(err)?
    };

    let now = match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(duration) => duration.as_secs(),
        Err(err) => Err(AppError::InternalServerError(err.to_string()))?
    };

    let exp = now as usize + AuthConfig::get_exp_claims_additional_sec();

    let iat = now as usize;

    let nbf = now as usize;

    let inf: HashMap<String, String> = vec![
        ("uuid".to_string(), user.uuid.to_string()),
        ("email".to_string(), user.email),
        ("username".to_string(), user.username),
    ].into_iter().collect();

    let jwt_claims = match JwtClaims::new(
        // Some(AuthConfig::get_aud_claims()),
        None,
        exp,
        Some(iat),
        Some(AuthConfig::get_iss_claims()),
        Some(nbf),
        Some(user.uuid.to_string()),
        Some(inf)
    ){
        Ok(result) => result,
        Err(err) => Err(
            AppError::AuthError(
                AppMsgError{
                    api_msg_error: "Internal Server Error".to_string(),
                    log_msg_error: err.to_string()
                }
            )
        )?
    };

    let result = match jwt_encode(AuthConfig::get_algorithm(), jwt_claims){
        Ok(result) => result,
        Err(err) => Err(
            AppError::AuthError(
                AppMsgError{
                    api_msg_error: "Internal Server Error".to_string(),
                    log_msg_error: err.to_string()
                }
            )
        )?
    };

    Ok(HttpResponse::Ok().json(json!({"token":result})))

}