use serde::{Serialize, Deserialize};
use crate::error_app::error_app::AppError;

#[derive(Serialize)]
pub struct PaginationFrom {
    pub page: u32,
    pub page_size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pagination {
    #[serde(default)]
    pub page: String,
    #[serde(default)]
    pub page_size: String,
}

impl Pagination {
    pub fn new(page: String, page_size: String) -> Result<PaginationFrom, AppError> {

        let page = match page.parse::<u32>(){
            Ok(page) => page,
            Err(e) => return Err(AppError::BadRequest(format!("Page must be a int: {}", e)))?
        };

        let page_size = match page_size.parse::<u32>(){
            Ok(page_size) => page_size,
            Err(e) => return Err(AppError::BadRequest(format!("Page size must be a int: {}", e)))?
        };

        match page.checked_sub(1) {
            Some(page) => (),
            None => return Err(AppError::BadRequest("Page must be greater than 0".to_string()))?
        }

        match page_size.checked_sub(1) {
            Some(page_size) => (),
            None => return Err(AppError::BadRequest("Page size must be greater than 0".to_string()))?
        };

        Ok(
            PaginationFrom{
                page,
                page_size,
            }
        )
    }
}


