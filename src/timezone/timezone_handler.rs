use actix_web::HttpResponse;
use chrono::Utc;
use chrono_tz::TZ_VARIANTS;
use crate::timezone::timezone_model::Timezone;

pub async fn timezone_get()-> HttpResponse {
    
    let utc_now = Utc::now();
    
    let timezones: Vec<Timezone> = TZ_VARIANTS
        .iter()
        .map(|tz| Timezone{
            name: tz.name().to_string(),
            now: utc_now.with_timezone(tz).to_rfc3339(),
        })
        .collect();
    
    HttpResponse::Ok().json(timezones)
}