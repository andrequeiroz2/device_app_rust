use log::error;
use sqlx::PgPool;
use crate::device::device_message_model::DeviceMessageSubscribe;
use crate::error_app::error_app::AppError;

pub async fn get_device_message_subscribe_query(
    pool: &PgPool,
)-> Result<Vec<DeviceMessageSubscribe>, AppError> {

    let result = sqlx::query_as!(
        DeviceMessageSubscribe,
        r#"
        SELECT
          d.uuid as device_uuid,
          m.uuid as message_uuid,
          m.topic,
          m.qos
        FROM devices d
        INNER JOIN messages m ON d.id = m.device_id
        WHERE
          d.device_condition_int = 0
          AND m.subscriber = true
          AND d.deleted_at IS NULL
          AND m.deleted_at IS NULL;
        "#,
    ).fetch_all(pool)
        .await
        .map_err(|error| 
            {
                error!("file: {}, line: {}, error: {}", file!(), line!(), error);
                AppError::DBError(error.to_string())
            }
        )?;
        

    Ok(result)
}