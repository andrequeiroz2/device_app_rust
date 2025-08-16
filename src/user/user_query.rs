use sqlx::{PgPool, QueryBuilder};
use crate::error_app::error_app::{AppError, AppMsgError};
use crate::user::user_model::{User, UserCreate, UserFilter, UserResponse};
use uuid::Uuid;

pub async fn user_count(
    pool: &PgPool,
    email: &String,
) -> Result<Option<i64>, AppError> {
    
    match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE email = $1 AND deleted_at IS NULL",
        email
    ).fetch_one(pool)
        .await {
            Ok(result)=> { 
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
            Err(err) =>Err(AppError::DBError(err.to_string()))?
    }
}

pub async fn get_user(
    pool: &PgPool,
    filter: &UserFilter,
) -> Result<UserResponse, AppError> {

    let mut builder = QueryBuilder::new(
        r#"
        SELECT id, uuid, username, email, created_at, updated_at, deleted_at
        FROM users
        WHERE deleted_at IS NULL
        "#,
    );

    if let Some(uuid) = &filter.uuid {
        builder.push(" AND uuid = ").push_bind(uuid);
    };

    if let Some(email) = &filter.email {
        builder.push(" AND email = ").push_bind(email);
    };

    let query = builder.build_query_as::<UserResponse>();

     let result = match query.fetch_all(pool).await{
        Ok(result) => {
            result.into_iter().next().ok_or(
                AppError::NotFound(
                    AppMsgError{
                        api_msg_error: "User not found".to_string(),
                        log_msg_error: format!("User not found: {:?}", &filter),
                    }

                )
            )?
        },
        Err(e) => Err(AppError::DBError(e.to_string()))?
    };

    Ok(result)
}

pub async fn get_user_full_row(
    pool: &PgPool,
    email: &String,
) -> Result<User, AppError>{

    match sqlx::query_as!(
        User,
        r#"
            SELECT id, uuid, username, password, email, created_at, updated_at, deleted_at
            FROM users
            WHERE deleted_at IS NULL AND email = $1
            "#,
        email,
    ).fetch_one(pool)
        .await {
        Ok(result) => Ok(result),
        Err(err) => Err(
            AppError::NotFound(
                AppMsgError{
                    api_msg_error: "User not found".to_string(),
                    log_msg_error: format!("{}, email: {}", err, email)
                }
            )
        )?
    }
}

pub async fn get_user_by_uuid(
    pool: &PgPool,
    user_uuid: &Uuid,
) -> Result<UserResponse, AppError> {

    match sqlx::query_as!(
        UserResponse,
        r#"
            SELECT uuid, username, email
            FROM users
            WHERE uuid = $1
            AND deleted_at IS NULL
        "#,
        user_uuid
    ).fetch_one(pool).await {
        Ok(result) => Ok(result),
        Err(e) => Err(
            AppError::NotFound(
                AppMsgError{
                    api_msg_error: "User not found".to_string(),
                    log_msg_error: format!("{}, user_uuid: {}", e, user_uuid)
                }
            )
        )?
    }
}

pub async fn post_user_query(
    pool: &PgPool,
    user: UserCreate,
    user_uuid: &Uuid,
    password_hash: &String,
) -> Result<UserResponse, AppError> {
    
    match sqlx::query_as!(
        UserResponse,
        r#"
            INSERT INTO users (uuid, username, email, password)
            VALUES ($1, $2, $3, $4)
            RETURNING uuid, username, email
        "#,
        user_uuid,
        user.username,
        user.email,
        password_hash,
    ).fetch_one(pool)
        .await {
        Ok(result) => Ok(result),
        Err(e) => Err(
            AppError::ConstraintViolation(
                AppMsgError{
                    api_msg_error: " email already registered".to_string(),
                    log_msg_error: e.to_string()
                }
            )
        )?
    }
}

pub async fn delete_user(
    pool: &PgPool,
    uuid: &Uuid,
) -> Result<(), AppError> {

    match sqlx::query!(
        "UPDATE users SET deleted_at = NOW() WHERE uuid = $1",
        uuid
    ).execute(pool)
        .await {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::DBError(e.to_string()))?
    }
}

pub async fn update_user(
    pool: &PgPool,
    username: &String,
    user_email: &String,
    user_uuid: &Uuid,
) -> Result<UserResponse, AppError> {

    match sqlx::query_as!(
        UserResponse,
        r#"
            UPDATE users SET username = $1, email = $2
            WHERE uuid = $3
            RETURNING uuid, username, email
        "#,
        username,
        user_email,
        user_uuid,
    ).fetch_one(pool)
        .await{
            Ok(result) => Ok(result),
            Err(e) => Err(AppError::DBError(e.to_string()))?
    }
}