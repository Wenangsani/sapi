pub mod handler;

use actix_web::web::{get, post, ServiceConfig, scope};

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("")
            .route("/", get().to(handler::home))
    );
}

pub fn gate_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/api")
            .route("/welcome/{name}", get().to(handler::welcome))
    );
}