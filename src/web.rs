pub use actix_web::web::Data as Data;
pub use actix_web::HttpRequest as Request;
pub use actix_web::HttpResponse as Response;
pub type Pool = Data<sqlx::MySqlPool>;

#[derive(Serialize)]
pub struct Warn<'a> {
    pub code: i32,
    pub message: &'a str,
}

pub mod datas {
    pub use actix_web::web::Json as Json;
    pub use actix_web::web::Form as Form;
    pub use actix_web::web::Path as Path;
}

pub mod types {
    use chrono::{DateTime, Utc};
    pub use String;
    pub type Int = i32;
    pub type Date = DateTime<Utc>;
}