use sqlx::{query_scalar, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;
use crate::broker::broker_model::{BrokerCreate, BrokerFilter, BrokerResponse, BrokerUpdate};
use crate::error_app::error_app::{AppError, AppMsgError};

pub async fn post_broker_query(
    pool: &PgPool,
    broker: BrokerCreate,
    broker_uuid: &Uuid,
) -> Result<BrokerResponse, AppError> {

    let result = sqlx::query_as!(
        BrokerResponse,
        r#"
        INSERT INTO brokers(
            uuid,
            host,
            port,
            client_id,
            version,
            keep_alive,
            clean_session,
            last_will_topic,
            last_will_message,
            last_will_qos,
            last_will_retain
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            uuid,
            host,
            port,
            client_id,
            version,
            version_text as "version_text!: String",
            keep_alive,
            clean_session,
            last_will_topic,
            last_will_message,
            last_will_qos,
            last_will_retain,
            connected,
            created_at,
            updated_at,
            deleted_at
        "#,
        broker_uuid,
        broker.host,
        broker.port,
        broker.client_id,
        broker.version,
        broker.keep_alive,
        broker.clean_session,
        broker.last_will_topic,
        broker.last_will_message,
        broker.last_will_qos,
        broker.last_will_retain,
    )
        .fetch_one(pool)
        .await
        .map_err(|error| AppError::ConstraintViolation(
            AppMsgError {
                api_msg_error: "Failed to insert or update broker".to_string(),
                log_msg_error: error.to_string(),
            }
        ))?;

    Ok(result)
}

pub async fn get_broker_query(
    pool: &PgPool,
    filter: &BrokerFilter,
) -> Result<Vec<BrokerResponse>, AppError> {

    let mut builder = QueryBuilder::new(
        "
        SELECT
        id,
        uuid,
        host,
        port,
        client_id,
        version,
        version_text,
        keep_alive,
        clean_session,
        last_will_topic,
        last_will_message,
        last_will_qos,
        last_will_retain,
        connected,
        created_at,
        updated_at,
        deleted_at
        FROM brokers
        WHERE deleted_at IS NULL
        ",
    );

    if let Some(id) = &filter.id {
        builder.push(" AND id = ").push_bind(id);
    };

    if let Some(uuid) = &filter.uuid {
        builder.push(" AND uuid = ").push_bind(uuid);
    };

    if let Some(host) = &filter.host{
        builder.push(" AND host = ").push_bind(host);
    };

    if let Some(port) = &filter.port{
        builder.push(" AND port = ").push_bind(port);
    };

    if let Some(connected) = &filter.connected{
        builder.push(" AND connected = ").push_bind(connected);
    }

    let query = builder.build_query_as::<BrokerResponse>();

    match query.fetch_all(pool).await{
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::DBError(e.to_string()))?
    }
}

pub async fn get_broker_with_uuid_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
) -> Result<BrokerResponse, AppError> {

    match sqlx::query_as!(
        BrokerResponse,
        r#"
        SELECT
            uuid,
            host,
            port,
            client_id,
            version,
            version_text as "version_text!: String",
            keep_alive,
            clean_session,
            last_will_topic,
            last_will_message,
            last_will_qos,
            last_will_retain,
            connected,
            created_at,
            updated_at,
            deleted_at
            FROM brokers
            WHERE deleted_at IS NULL
            AND uuid = $1
        "#,
        broker_uuid
    ).fetch_one(pool).await{
        Ok(result) => Ok(result),
        Err(err) => Err(
            AppError::NotFound(
                AppMsgError{
                    api_msg_error: "Broker not found".to_string(),
                    log_msg_error: format!("{}, uuid: {}", err, broker_uuid)
                }
            )
        )?
    }
}

pub async fn delete_broker_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
) -> Result<(), AppError> {

    match sqlx::query!(
        "UPDATE brokers SET deleted_at = NOW() WHERE uuid = $1",
        broker_uuid
    ).execute(pool)
        .await {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::DBError(e.to_string()))?
    }
}

pub async fn get_broker_update_check_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
    broker_update: &BrokerUpdate,
)-> Result<(), AppError> {

    let exists: Option<Uuid> = query_scalar!(
        r#"
        SELECT uuid
        FROM brokers
        WHERE uuid <> $1
          AND (
                ($2::text IS NOT NULL AND host = $2)
             OR ($3::int  IS NOT NULL AND port = $3)
          )
        LIMIT 1
        "#,
        broker_uuid,
        broker_update.host,
        broker_update.port
    ).fetch_optional(pool)
        .await
        .map_err(|e| AppError::DBError(e.to_string()))?;

    if exists.is_some() {
        Err(
            AppError::ConstraintViolation(
                AppMsgError{
                    api_msg_error: "Host or Port already registered".to_string(),
                    log_msg_error: format!(
                        "Host or Port already registered, host: {:?}, port: {:?}",
                        broker_update.host,
                        broker_update.port
                    )
                }
            )
        )?
    }
    Ok(())
}

pub async fn put_broker_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
    broker_update: &BrokerUpdate,
) -> Result<BrokerResponse, AppError> {

    match sqlx::query_as!(
        BrokerResponse,
        r#"
        UPDATE brokers SET
            host = $1,
            port = $2,
            client_id = $3,
            version = $4,
            keep_alive = $5,
            clean_session = $6,
            last_will_topic = $7,
            last_will_message = $8,
            last_will_qos = $9,
            last_will_retain = $10,
            connected = $11
        WHERE uuid = $12
        RETURNING
            uuid,
            host,
            port,
            client_id,
            version,
            version_text as "version_text!: String",
            keep_alive,
            clean_session,
            last_will_topic,
            last_will_message,
            last_will_qos,
            last_will_retain,
            connected,
            created_at,
            updated_at,
            deleted_at
          "#,
        broker_update.host,
        broker_update.port,
        broker_update.client_id,
        broker_update.version,
        broker_update.keep_alive,
        broker_update.clean_session,
        broker_update.last_will_topic,
        broker_update.last_will_message,
        broker_update.last_will_qos,
        broker_update.last_will_retain,
        broker_update.connected,
        broker_uuid
    ).fetch_one(pool).await{
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::DBError(e.to_string()))?
    }
}