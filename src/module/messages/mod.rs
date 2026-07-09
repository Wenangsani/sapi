use actix_web::web::{self, scope, ServiceConfig};

pub mod handler;

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/messages")
            .route("", web::get().to(handler::page))
    );
}

pub fn gate_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/messages")
            .route("/conversations", web::get().to(handler::list_conversations))
            .route("/conversations", web::post().to(handler::create_conversation))
            .route("/conversations/{id}", web::get().to(handler::get_messages))
            .route("/conversations/{id}", web::post().to(handler::send_message))
            .route("/conversations/{id}/read", web::post().to(handler::mark_read))
            .route("/users/search", web::get().to(handler::search_users))
            .route("/{id}", web::delete().to(handler::delete_message))
    );
}