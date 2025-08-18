use sqlx::PgPool;
use crate::device::device_model::{DeviceCreate, DeviceResponse};
use crate::error_app::error_app::AppError;

pub async fn post_device_query(
    pool: &PgPool,
    device: DeviceCreate
) -> Result<DeviceResponse, AppError>{

    match sqlx::query_as!(
        DeviceResponse,
        "INSERT INTO devices (uuid, name, topic) VALUES ($1, $2, $3) RETURNING name, topic",
        uuid::Uuid::new_v4(),
        device.name,
        device.topic
    ).fetch_one(pool)
        .await{
        Ok(device) => Ok(device),
        Err(e) => Err(AppError::BadRequest(e.to_string()))?
    }
}