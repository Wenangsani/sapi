pub use actix_web::Error;
pub use actix_web::HttpRequest as Request;
pub use actix_web::HttpResponse as Response;
pub use serde_json::Value;
pub use uuid::Uuid;
pub use actix_session::Session as Session;
pub use actix_web::cookie::Cookie as Cookie;


pub type Pool = actix_web::web::Data<sqlx::MySqlPool>;

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

pub mod data {
    pub use actix_web::web::Data;
    pub use actix_web::web::Form;
    pub use actix_web::web::Json;
    pub use actix_web::web::Path;
}

pub mod types {
    use chrono::{DateTime, Utc};
    pub use String;
    pub type Int = i32;
    pub type Date = DateTime<Utc>;
}
