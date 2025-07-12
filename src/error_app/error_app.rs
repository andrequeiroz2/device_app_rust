use actix_web::{error, http::StatusCode, HttpResponse};
use serde::Serialize;
use log::error;
use std::fmt;
use sqlx::error::Error as SQLxError;
use jwt_lib::error::AuthError;

#[derive(Debug, Serialize)]
pub struct AppMsgError {
    pub api_msg_error: String,
    pub log_msg_error: String,
}

#[derive(Debug, Serialize)]
pub enum AppError{
    BadRequest(String),
    NotFound(AppMsgError),
    ConstraintViolation(AppMsgError),
    UnprocessableEntity(AppMsgError),
    DBError(String),
    ActixError(String),
    ScryptError(AppMsgError),
    Unauthorized(AppMsgError),
    AuthError(AppMsgError),
    InternalServerError(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse{
    error_message: String
}

impl AppError{
    fn error_response(&self) -> String{
        match self{
            AppError::BadRequest(msg) => {
                error!("Bad request occurred: {}", msg);
                msg.into()
            }

            AppError::NotFound(msg) => {
                error!("Not found occurred: {}", msg.log_msg_error);
                msg.api_msg_error.clone()
            }

            AppError::DBError(msg) => {
                error!("DB error occurred: {}", msg);
                "internal server error".into()
            }

            AppError::ConstraintViolation(msg) => {
                error!("Constraint violation occurred: {}", msg.log_msg_error);
                msg.api_msg_error.clone()
            }

            AppError::ActixError(msg) => {
                error!("Actix error occurred: {}", msg);
                "Actix error".into()
            }

            AppError::UnprocessableEntity(msg) => {
                error!("Unprocessable entity occurred: {}", msg.log_msg_error);
                msg.api_msg_error.clone()
            }

            AppError::ScryptError(msg) => {
                error!("Scrypt error occurred: {}", msg.log_msg_error);
                msg.api_msg_error.clone()
            }

            AppError::Unauthorized(ms)=> {
                error!("Unauthorized error occurred: {}", ms.log_msg_error);
                "Unauthorized".into()
            }

            AppError::AuthError(msg) => {
                error!("Auth error occurred: {}", msg.log_msg_error);
                "Auth error".into()
            }

            AppError::InternalServerError(msg) => {
                error!("Internal server error occurred: {}", msg);
                "Internal server error".into()
            }
        }
    }
}

impl error::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_msg)=>StatusCode::BAD_REQUEST,
            AppError::NotFound(_msg)=>StatusCode::NOT_FOUND,
            AppError::ConstraintViolation(_msg)=>StatusCode::CONFLICT,
            AppError::UnprocessableEntity(_msg)=>StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Unauthorized(_msg)=>StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error_message: self.error_response(),
        })
    }
}

impl fmt::Display for AppError{
    fn fmt(&self, f: &mut fmt::Formatter)-> Result<(), fmt::Error>{
        write!(f, "{}", self)
    }
}

impl From<actix_web::error::Error> for AppError {
    fn from(err: actix_web::error::Error) -> Self{
        AppError::ActixError(err.to_string())
    }
}
impl From<SQLxError> for AppError{
    fn from(err: SQLxError)-> Self{
        AppError::DBError(err.to_string())
    }
}

impl From<AuthError> for AppError{
    fn from(err: AuthError)-> Self{
        AppError::AuthError(
            AppMsgError{
                api_msg_error: "Internal server error".into(),
                log_msg_error: err.to_string()
            }
        )
    }
}