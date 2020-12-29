use sqlx::MySqlPool;
use actix_web::web::Data;

pub use actix_web::HttpRequest as Request;
pub use actix_web::HttpResponse as Response;
pub use actix_web::web::Json as Json;

pub type Pool = Data<MySqlPool>;

#[derive(Serialize, Debug)]
pub struct Warn {
    pub error: i32,
    pub content: String,
}
