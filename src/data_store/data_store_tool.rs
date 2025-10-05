use chrono::{DateTime, Utc};
use mongodb::bson::DateTime as BsonDateTime;
use crate::error_app::error_app::{AppError, AppMsgInfError};

pub fn bson_to_chrono(bson_dt: &BsonDateTime) -> Result<DateTime<Utc>, AppError> {
    let millis = bson_dt.timestamp_millis();
    let secs = millis / 1000;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;

    let data_time = DateTime::<Utc>::from_timestamp(secs, nsecs).ok_or_else(|| {
        AppError::MongoDBError(AppMsgInfError {
            file: file!().to_string(),
            line: line!(),
            api_msg_error: "Internal server error".into(),
            log_msg_error: "Error converting bson to chrono".into(),
        })
    })?;
    
    Ok(data_time)
}