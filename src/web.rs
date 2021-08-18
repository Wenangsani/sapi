pub use actix_web::Error;
pub use actix_web::HttpRequest as Request;
pub use actix_web::HttpResponse as Response;
pub use uuid::Uuid;

pub type Pool = actix_web::web::Data<sqlx::MySqlPool>;

#[derive(Serialize)]
pub struct Warning<'a> {
    pub message: &'a str,
}

pub mod datas {
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
