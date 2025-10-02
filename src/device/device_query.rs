use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use crate::device::device_model::{Device, DeviceCreate, DeviceFilter};
use crate::error_app::error_app::{AppError};
use crate::device::device_message_model::{DeviceMessage};

pub async fn get_device_filter(
    pool: &PgPool,
    filter: &DeviceFilter
) -> Result<Option<Device>, AppError> {

    let mut builder = QueryBuilder::new(
        r#"
        SELECT
            id,
            uuid,
            user_id,
            name,
            device_type_int,
            device_type_text,
            border_type_int,
            border_type_text,
            device_condition_int,
            device_condition_text,
            mac_address,
            created_at,
            updated_at,
            deleted_at
        FROM devices
        WHERE deleted_at IS NULL
        "#,
    );

    if let Some(uuid) = &filter.uuid {
        builder.push(" AND uuid = ").push_bind(uuid);
    };

    if let Some(mac_address) = &filter.mac_address {
        builder.push(" AND mac_address = ").push_bind(mac_address);
    };

    let query = builder.build_query_as::<Device>();

    let opt = query
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    Ok(opt)
}

pub async fn post_device_message_query(
    pool: &PgPool,
    device: DeviceCreate,
) -> Result<(Device, DeviceMessage), AppError>{

    let mut tx: Transaction<'_, Postgres> = pool.begin().await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    // Insert device
    let insert_device = sqlx::query_as!(
        Device,
        r#"
        INSERT INTO
        devices (
         uuid,
         user_id,
         name,
         device_type_int,
         device_type_text,
         border_type_int,
         border_type_text,
         device_condition_int,
         device_condition_text,
         mac_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
        id,
        uuid,
        user_id,
        name,
        device_type_int,
        device_type_text,
        border_type_int,
        border_type_text,
        device_condition_int,
        device_condition_text,
        mac_address,
        created_at,
        updated_at,
        deleted_at
        "#,
        device.uuid,
        device.user_id,
        device.name,
        device.device_type_int,
        device.device_type_text,
        device.border_type_int,
        device.border_type_text,
        device.device_condition_int,
        device.device_condition_text,
        device.mac_address,
    )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    // Insert message
    let inserted_message = sqlx::query_as!(
        DeviceMessage,
        r#"
        INSERT INTO
        messages (
          uuid,
          device_id,
          topic,
          payload,
          qos,
          retained,
          publisher,
          subscriber,
          scale,
          command_start,
          command_end,
          command_last,
          command_last_time
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING
        id,
        uuid,
        device_id,
        topic,
        payload,
        qos,
        retained,
        publisher,
        subscriber,
        scale,
        command_start,
        command_end,
        command_last,
        command_last_time,
        created_at,
        updated_at,
        deleted_at
        "#,
        device.message.uuid,
        insert_device.id,
        device.message.topic,
        device.message.payload,
        device.message.qos,
        device.message.retained,
        device.message.publisher,
        device.message.subscriber,
        device.message.scale,
        device.message.command_start,
        device.message.command_end,
        device.message.command_last,
        device.message.command_last_time,
    )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    //commit
    tx.commit().await.map_err(|e| AppError::DBError(e.to_string()))?;

    Ok((insert_device, inserted_message))
}