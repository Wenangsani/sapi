use uuid::Uuid;

pub use actix_web::HttpRequest as Request;
pub use actix_web::HttpResponse as Response;
pub use actix_web::Error as Error;

pub type Pool = actix_web::web::Data<sqlx::MySqlPool>;

#[derive(Serialize)]
pub struct Warning<'a> {
    pub code: i32,
    pub message: &'a str,
}

pub mod datas {
    pub use actix_web::web::Json as Json;
    pub use actix_web::web::Form as Form;
    pub use actix_web::web::Path as Path;
    pub use actix_web::web::Data as Data;
}

pub mod types {
    use chrono::{DateTime, Utc};
    pub use String;
    pub type Int = i32;
    pub type Date = DateTime<Utc>;
}

// Define HTTP actor
#[derive(Clone, Debug)]
pub struct WsConn {
    pub id: Uuid
}

impl WsConn {

    // Create new socket connection
    pub fn new() -> WsConn {

        return WsConn {
            id: Uuid::new_v4()
        };
    }

    pub fn send(&mut self) {

        println!("self: {:?}", self);

        // ctx.text("taa".to_owned());
    }
}