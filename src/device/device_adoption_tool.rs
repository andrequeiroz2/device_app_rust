use uuid::Uuid;

pub fn device_compose_topic(user_uuid: &Uuid, device_uuid: &Uuid, device_name: &str) -> String{
    format!("{}/{}/{}", user_uuid, device_uuid, device_name)
}