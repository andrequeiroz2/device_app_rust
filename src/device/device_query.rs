use log::error;
use std::vec::Vec;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use crate::device::device_model::{Device, DeviceCreate, DeviceFilter, DevicePaginationFilter};
use crate::error_app::error_app::{AppError};
use crate::device::device_message_model::{DeviceMessage, DeviceMessageCreateResponse, DeviceScale};
use crate::paginate::paginate_model::Pagination;

pub async fn get_device_topic_filter_query(
    pool: &PgPool,
    device_filter: &Vec<i32>,
)-> Result<Vec<DeviceMessageCreateResponse>, AppError>{
    
    match sqlx::query_as!(
        DeviceMessageCreateResponse,
        r#"
            SELECT
                m.uuid,
                d.uuid as "device_uuid!",
                m.topic,
                m.qos,
                m.retained,
                m.publisher,
                m.subscriber,
                m.command_start,
                m.command_end,
                m.command_last,
                m.command_last_time,
                m.created_at,
                m.updated_at,
                m.deleted_at
            FROM messages m
            INNER JOIN devices d ON m.device_id = d.id
            WHERE m.device_id = ANY($1)
            AND m.deleted_at IS NULL
            AND d.deleted_at IS NULL
        "#,
        device_filter as &[i32]
    ).fetch_all(pool).await{
        Ok(result) => Ok(result),
        Err(err) => Err(AppError::DBError(format!("{}", err)))?,
    }
}

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
            board_type_int,
            board_type_text,
            sensor_type,
            actuator_type,
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
        .map_err(|e|
            {
                error!("file: {}, line: {}, error: {}", file!(), line!(), e);
                AppError::DBError(e.to_string())
            })?;

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
        .map_err(|e| {
            error!("file: {}, line: {}, error: {}", file!(), line!(), e);
            AppError::DBError(e.to_string())
        })?;

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
         board_type_int,
         board_type_text,
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
        board_type_int,
        board_type_text,
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
        device.board_type_int,
        device.board_type_text,
        sensor_type_str,
        actuator_type_str,
        device.device_condition_int,
        device.device_condition_text,
        device.mac_address,
    )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DBError(format!("file: {}, line: {}, error: {}", file!(), line!(), e.to_string())))?;

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
        .map_err(|e|
            {
                error!("file: {}, line: {}, error: {}", file!(), line!(), e);
                AppError::DBError(e.to_string())
            }
        )?;

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
            .map_err(|e|
                {
                    error!("file: {}, line: {}, error: {}", file!(), line!(), e);
                    AppError::DBError(e.to_string())
                }
            )?;

            inserted_scale.push(scale);
        }
    }

    //commit
    tx.commit().await.map_err(|e|
        {
            error!("file: {}, line: {}, error: {}", file!(), line!(), e);
            AppError::DBError(e.to_string())
        }
    )?;

    Ok((inserted_device, inserted_message, inserted_scale))
}

pub async fn get_devices_owned_by_user(
    pool: &PgPool,
    user_id: i32,
    pagination: &DevicePaginationFilter,
) -> Result<Vec<Device>, AppError>{

    let mut builder = QueryBuilder::new(
        r#"
        SELECT
            id,
            uuid,
            user_id,
            name,
            device_type_int,
            device_type_text,
            board_type_int,
            board_type_text,
            sensor_type,
            actuator_type,
            device_condition_int,
            device_condition_text,
            mac_address,
            created_at,
            updated_at,
            deleted_at
        FROM devices
        WHERE deleted_at IS NULL "#,
    );

    builder.push(" AND user_id = ");
    builder.push_bind(user_id);

    println!("SQL: {}", builder.sql());

    //Pagination
    let page: String;
    let page_size: String;

    if pagination.pagination.page.is_empty(){
        page = "1".to_string();
    }else{
        page = pagination.pagination.page.clone();
    };

    if pagination.pagination.page_size.is_empty(){
        page_size = "10".to_string();
    }else{
        page_size = pagination.pagination.page_size.clone();
    };

    let pagination = match Pagination::new(
        page,
        page_size,
    ){
        Ok(result) => result,
        Err(err) => Err(err)?
    };

    let offset = (pagination.page.saturating_sub(1) * pagination.page_size) as i64;

    builder.push(" ORDER BY id ASC ");
    builder.push(" LIMIT ").push_bind(pagination.page_size as i64);
    builder.push(" OFFSET ").push_bind(offset);

    let query = builder.build_query_as::<Device>();

    let devices =  match query.fetch_all(pool).await{
        Ok(result) => result,
        Err(err) => {
            log::error!("file: {}, line: {}, error: {}", file!(), line!(), err);
            Err(AppError::DBError(err.to_string()))? }
    };
        Ok(devices)
}


pub async fn get_device_count_total_owned_user(pool: &PgPool, user_id: i32) -> Result<i64, AppError> {
    match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM devices WHERE deleted_at IS NULL and user_id = $1",
        user_id
    ).fetch_one(pool)
        .await{
        Ok(result) => {
            match result {
                Some(count) => Ok(count),
                None => Ok(0)
            }
        },
        Err(error) => Err(AppError::DBError(error.to_string()))?
    }
}