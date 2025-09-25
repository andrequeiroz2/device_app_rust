use jwt_lib::jwt_decode;
use scrypt::password_hash::{PasswordHash, PasswordVerifier};
use scrypt::Scrypt;
use log::{info, log};
use crate::auth::auth_config::AuthConfig;
use crate::auth::auth_model::{MyClaim, Token};
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

pub async fn token_info(token: String) -> Result<Token, AppError>{

    let algorithm = AuthConfig::get_algorithm();

    let claim = match jwt_decode(algorithm, token.clone()){
        Ok(claim)=> claim,
        Err(err) => Err(
            AppError::Unauthorized(
                AppMsgError{
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: format!("file: {}, line: {}, {}, algorithm: {}, token: {}", file!(), line!(), err, algorithm, token),
                }
            )
        )?
    };


    let claim_value = match serde_json::to_value(&claim){
        Ok(result) => result,
        Err(err) => {
            return Err(AppError::Unauthorized(
                AppMsgError {
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: format!("file: {}, line: {}, {}, algorithm: {}, token: {}", file!(), line!(), err, algorithm, token),
                }
            ))?;
        }
    };

    let my_claim: MyClaim = match serde_json::from_value(claim_value){
        Ok(result) => result,
        Err(err) => {
            return Err(AppError::Unauthorized(
                AppMsgError {
                    api_msg_error: "Unauthorized".to_string(),
                    log_msg_error: format!("file: {}, line: {}, {}, algorithm: {}, token: {}", file!(), line!(), err, algorithm, token),
                }
            ))?;
        }
    };

    Ok(
        Token::build(my_claim)
    )

}