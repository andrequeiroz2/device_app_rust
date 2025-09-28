use sqlx::{query_scalar, PgPool, QueryBuilder};
use uuid::Uuid;
use crate::broker::broker_model::{BrokerCreate, BrokerFilter, BrokerPaginationResponse, BrokerResponse, BrokerUpdate};
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::paginate::paginate_model::Pagination;

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
) -> Result<BrokerPaginationResponse, AppError> {

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

    //Pagination
    let page: String;
    let page_size: String;

    if filter.pagination.page.is_empty(){
        page = "1".to_string();
    }else{
        page = filter.pagination.page.clone();
    };

    if filter.pagination.page_size.is_empty(){
        page_size = "10".to_string();
    }else{
        page_size = filter.pagination.page_size.clone();
    };

    let pagination = match Pagination::new(
        page,
        page_size,
    ){
        Ok(result) => result,
        Err(err) => Err(err)?
    };

    let offset = (pagination.page.saturating_sub(1) * pagination.page_size) as i64;

    builder.push(" ORDER BY host ASC ");
    builder.push(" LIMIT ").push_bind(pagination.page_size as i64);
    builder.push(" OFFSET ").push_bind(offset);

    let query = builder.build_query_as::<BrokerResponse>();

    let brokers = match query.fetch_all(pool).await{
        Ok(result) => result,
        Err(e) => Err(AppError::DBError(e.to_string()))?
    };

    let brokers_count = match get_broker_count_total_query(pool).await{
        Ok(result) => result,
        Err(e) => Err(e)?
    };

    let result = BrokerPaginationResponse::new(
        brokers,
        brokers_count,
        pagination.page,
        pagination.page_size
    );

    Ok(result)

}

pub async fn get_broker_with_uuid_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
) -> Result<Option<BrokerResponse>, AppError> {

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
    ).fetch_optional(pool).await{
        Ok(result) => Ok(result),
        Err(err) => Err(
            AppError::DBError(err.to_string()))?
    }
}

pub async fn delete_broker_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
) -> Result<(), AppError> {

    match sqlx::query!(
        "DELETE FROM brokers WHERE uuid = $1",
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
)-> Result<i64, AppError> {

    match query_scalar!(
        r#"
        SELECT COUNT(*) FROM brokers
        WHERE uuid <> $1
            AND port = $2
            AND deleted_at IS NULL
        "#,
        broker_uuid,
        broker_update.port
    ).fetch_one(pool)
        .await{
            Ok(result) =>{
                match result {
                    Some(result) => Ok(result),
                    None => Ok(0)
                }
            }
            Err(e) => Err(AppError::DBError(e.to_string()))?
        }
}

pub async fn put_broker_state_query(
    pool: &PgPool,
    broker_uuid: &Uuid,
    connected: bool,
) -> Result<(), AppError> {

    let query = r#"
        UPDATE brokers
        SET connected = $1
        WHERE uuid = $2
    "#;

    match sqlx::query(query)
        .bind(connected)
        .bind(broker_uuid)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::DBError(e.to_string())),
    }
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

pub async fn get_broker_count_query(
    pool: &PgPool,
    port: i32
) -> Result<Option<i64>, AppError> {

    match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM brokers WHERE port = $1 AND deleted_at IS NULL",
        port
    ).fetch_one(pool)
        .await {
        Ok(result) => {
            match result {
                Some(count) => {
                    if count == 0 {
                        Ok(None)
                    }else {
                        Ok(Some(count))
                    }
                },
                None => Ok(None),
            }
        },
        Err(e) => Err(AppError::DBError(e.to_string()))?
    }
}

pub async fn get_broker_count_total_query(pool: &PgPool) -> Result<i64, AppError> {
    match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM brokers WHERE deleted_at IS NULL"
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