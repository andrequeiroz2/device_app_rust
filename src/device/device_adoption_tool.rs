use log::error;
use uuid::Uuid;
use crate::error_app::error_app::AppError;

pub struct DecomposeTopic {
    pub user_uuid: Uuid,
    pub device_uuid: Uuid,
    pub device_name: String,
}

pub fn device_compose_topic(user_uuid: &Uuid, device_uuid: &Uuid, device_name: &str) -> String{
    format!("{}/{}/{}", user_uuid, device_uuid, device_name)
}

pub fn device_decompose_topic(topic: &str) -> Result<DecomposeTopic, AppError> {

    let topic_split: Vec<&str> = topic.split("/").collect();

    if topic_split.len() == 3 {
        {};
    } else {
        error!("file: {}, line: {}, Invalid topic, topic: {}", file!(), line!(), topic);
        return Err(AppError::BadRequest("Invalid topic".to_string()))
    }
    
    let user_uuid = match Uuid::parse_str(topic_split[0]){
        Ok(uuid) => uuid,
        Err(_) => return Err(AppError::BadRequest("Invalid user_uuid".to_string()))
    };

    let device_uuid = match Uuid::parse_str(topic_split[1]){
        Ok(uuid) => uuid,
        Err(_) => return Err(AppError::BadRequest("Invalid device_uuid".to_string()))
    };

    let device_name = topic_split[2].to_string();

    Ok(
        DecomposeTopic{
            user_uuid,
            device_uuid,
            device_name
        }
    )
}