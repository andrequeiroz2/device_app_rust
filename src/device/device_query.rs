use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use crate::device::device_model::{Device, DeviceCreate, DeviceFilter};
use crate::error_app::error_app::{AppError};
use crate::device::device_message_model::{DeviceMessage, DeviceScale};

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
    device: &DeviceCreate,
    topic_compose: String
) -> Result<(Device, DeviceMessage, Vec<DeviceScale>), AppError>{

    let sensor_type_str = device.sensor_type.clone();
    let actuator_type_str = device.actuator_type.clone();

    let mut tx: Transaction<'_, Postgres> = pool.begin().await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    // Insert device
    let inserted_device = sqlx::query_as!(
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
         sensor_type,
         actuator_type,
         device_condition_int,
         device_condition_text,
         mac_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
        id,
        uuid,
        user_id,
        name,
        device_type_int,
        device_type_text,
        border_type_int,
        border_type_text,
        sensor_type,
        actuator_type,
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
        sensor_type_str,
        actuator_type_str,
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
          qos,
          retained,
          publisher,
          subscriber,
          command_start,
          command_end,
          command_last,
          command_last_time
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
        id,
        uuid,
        device_id,
        topic,
        qos,
        retained,
        publisher,
        subscriber,
        command_start,
        command_end,
        command_last,
        command_last_time,
        created_at,
        updated_at,
        deleted_at
        "#,
        device.message.uuid,
        inserted_device.id,
        topic_compose,
        device.message.qos,
        device.message.retained,
        device.message.publisher,
        device.message.subscriber,
        device.message.command_start,
        device.message.command_end,
        device.message.command_last,
        device.message.command_last_time,
    )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    // Insert scale
    let mut inserted_scale: Vec<DeviceScale> = Vec::new();

    if let Some(scale_list) = device.scale.clone() {
        for scale_item in scale_list {
            let scale = sqlx::query_as!(
            DeviceScale,
            r#"
            INSERT INTO 
            scales (
                uuid,
                device_id,
                metric,
                unit
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
            id,
            uuid,
            device_id,
            metric,
            unit,
            created_at,
            updated_at,
            deleted_at
            "#,
                
            scale_item.uuid,
            inserted_device.id,
            scale_item.metric,
            scale_item.unit
        )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::DBError(e.to_string()))?;

            inserted_scale.push(scale);
        }
    }

    //commit
    tx.commit().await.map_err(|e| AppError::DBError(e.to_string()))?;

    Ok((inserted_device, inserted_message, inserted_scale))
}