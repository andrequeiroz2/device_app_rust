use actix_web::{web, HttpResponse};
use crate::error_app::error_app::{AppError, AppMsgError};
use uuid::Uuid;
use crate::state::AppState;
use crate::user::user_model::{UserCreate, UserFilter, UserUpdate};
use crate::user::user_query::{delete_user, get_user, get_user_by_uuid, post_user_query, update_user, user_count};
use crate::user::user_tool::get_password_hash;

pub async fn user_create(
    user: web::Json<UserCreate>,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AppError>{

    if user.password != user.confirm_password{
        Err(
            AppError::UnprocessableEntity(
                AppMsgError{
                    api_msg_error: "Password and confirm_password do not match".to_string(),
                    log_msg_error: format!(
                        "Password and confirm password do not match, password: {}, \
                        confirm_password: {}",
                        user.password,
                        user.confirm_password
                    ),
                }
            )
        )?
    }

    /* Check user exists */
    let _: Result<(), AppError> = match user_count(&app_state.db, &user.email)
        .await {
        Ok(result) => match result {
            Some(result) => {
                println!("result: {}", result);
                Err(
                    AppError::ConstraintViolation(
                        AppMsgError {
                            api_msg_error: format!("email already registered: {}", user.email),
                            log_msg_error: format!("email already registered: {}", user.email),
                        }
                    )
                )?
            },
            None => Ok(())
        },
        Err(e) => Err(e)?
    };

    let password_hash = match get_password_hash(&user.password){
        Ok(password_hash) => password_hash,
        Err(err) => Err(err)?
    };

    post_user_query(&app_state.db, user.into(), &Uuid::new_v4(), &password_hash)
        .await
        .map(|user| HttpResponse::Ok().json(user))
}

pub async fn user_get_filter(
    filter: web::Query<UserFilter>,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AppError>{

    if filter.is_empty(){
        Err(
            AppError::BadRequest(
                "At least one filter parameter must be provided: 'uuid' or 'email'".to_string()
            )
        )?
    }

    get_user(&app_state.db, &filter)
        .await
        .map(|user| HttpResponse::Ok().json(user))
}

pub async fn user_soft_delete(
    user_uuid: web::Path<Uuid>,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AppError>{

    let user_uuid = user_uuid.into_inner();

    let filter = UserFilter{
        uuid: Some(user_uuid),
        email: None,
    };

    match get_user(&app_state.db, &filter).await{
        Ok(user) => user,
        Err(e) => Err(e)?
    };

    delete_user(&app_state.db, &user_uuid)
        .await
        .map(|_| HttpResponse::NoContent().finish())
}

pub async fn user_update(
    user_uuid: web::Path<Uuid>,
    user: web::Json<UserUpdate>,
    app_state: web::Data<AppState>
) -> Result<HttpResponse, AppError>{

    if user.is_empty(){
        Err(
            AppError::BadRequest(
                "At least one json field must be provided: 'username' or 'email'".to_string()
            )
        )?
    }

    let user_uuid = user_uuid.into_inner();

    let user_check = match get_user_by_uuid(&app_state.db, &user_uuid).await{
        Ok(result) => result,
        Err(e) => Err(e)?
    };

    let username = if let Some(username) = user.username() {
        &username.to_lowercase()
    }else{
        &user_check.username.to_lowercase()
    };

    let email = if let Some(email) = user.email() {
        &email.to_lowercase()
    }else{
        &user_check.email.to_lowercase()
    };

    update_user(&app_state.db, username, email, &user_uuid)
        .await
        .map(|user| HttpResponse::Ok().json(user))

}