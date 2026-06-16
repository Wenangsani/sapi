pub mod handler;

use actix_web::web::{get, post, ServiceConfig, scope};

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/auth")
            .route("/login",         get().to(handler::loginpage))
            .route("/api/login",    post().to(handler::login))
            .route("/register",      get().to(handler::registerpage))
            .route("/api/register", post().to(handler::register))
            .route("/logout",        get().to(handler::logout))
    );
}