use scrypt::password_hash::{PasswordHash, PasswordVerifier};
use scrypt::Scrypt;
use log::info;
use crate::error_app::error_app::{AppError, AppMsgError};

pub fn verify_password(password: &str, password_hash: &str) -> Result<(), AppError>{

    let parsed_hash = match PasswordHash::new(password_hash){
        Ok(hash)=> hash,
        Err(err)=> Err(
            AppError::Unauthorized(
                AppMsgError{
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: format!("{}, password: {}, password_hash: {}", err, password, password_hash)

                }
            )
        )?
    };
    info!("verify_password: parsed_hash: {:?}, password {:?}: ", parsed_hash, password);

    if Scrypt.verify_password(password.as_bytes(), &parsed_hash).is_err(){
        Err(
            AppError::Unauthorized(
                AppMsgError{
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: format!("Invalid password, password: {}, password_hash: {}", password, password_hash)
                }
            )
        )?
    } else {
        Ok(())
    }
}