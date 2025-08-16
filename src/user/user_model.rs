use serde::{Serialize, Deserialize};
use actix_web::web;
use chrono::Utc;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct User{
    pub id: i32,
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub deleted_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserCreate {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

impl From<web::Json<UserCreate>> for UserCreate {
    fn from(user: web::Json<UserCreate>) -> Self {
        UserCreate{
            username: user.username.to_lowercase().clone(),
            email: user.email.to_lowercase().clone(),
            password: user.password.clone(),
            confirm_password: user.confirm_password.clone(),
        }
    }
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct UserResponse {
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UserFilter {
    pub uuid: Option<Uuid>,
    pub email: Option<String>,
}

impl From<web::Query<UserFilter>> for UserFilter {
    fn from(filter: web::Query<UserFilter>) -> Self {
        UserFilter{
            uuid: filter.uuid.clone(),
            email: filter.email.clone(),
        }
    }
}
impl UserFilter {
    pub fn is_empty(&self) -> bool {
        self.uuid.is_none() && self.email.is_none()
    }
}

#[derive(Debug, Deserialize)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
}
impl From<web::Json<UserUpdate>> for UserUpdate {
    fn from(user: web::Json<UserUpdate>) -> Self {
        UserUpdate{
            username: user.username.clone(),
            email: user.email.clone(),
        }
    }
}
impl UserUpdate {
    pub fn is_empty(&self) -> bool {
        self.username.is_none() && self.email.is_none()
    }
    pub fn username(&self) -> Option<&String> {
        self.username.as_ref()
    }
    pub fn email(&self) -> Option<&String> {
        self.email.as_ref()
    }
}