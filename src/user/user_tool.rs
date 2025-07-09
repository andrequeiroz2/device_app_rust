use scrypt::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Scrypt,
};
use crate::error_app::error_app::{AppError, AppMsgError};

pub fn get_password_hash(password: &String) -> Result<String, AppError> {

    let salt = SaltString::generate(&mut OsRng);

    match Scrypt.hash_password(password.as_bytes(), &salt){
        Ok(hash) => Ok(hash.to_string()),
        Err(err)=> Err(
            AppError::ScryptError(
                AppMsgError{
                    api_msg_error: "Scrypt error occurred".to_string(),
                    log_msg_error: err.to_string()
                }
            )
        )?
    }
}