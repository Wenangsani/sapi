pub mod handler;

use actix_web::web::{get, ServiceConfig, scope};

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/test")
            .route("/api",    get().to(handler::api))
            .route("/socket", get().to(handler::socket))
            .route("/sse",    get().to(handler::sse))
    );
}