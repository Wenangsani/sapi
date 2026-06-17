use actix_web::web::ServiceConfig;

pub mod handler;

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/forum")
            // Halaman HTML
            .route("", actix_web::web::get().to(handler::page_forum))
            .route("/thread/{id}", actix_web::web::get().to(handler::page_thread))
            // API publik
            .route("/api/threads", actix_web::web::get().to(handler::list_threads))
            .route("/api/threads/{id}", actix_web::web::get().to(handler::get_thread))
            .route("/api/tags", actix_web::web::get().to(handler::list_tags))
            .route(
                "/api/threads/{id}/verify-password",
                actix_web::web::post().to(handler::verify_password),
            ),
    );
}

pub fn gate_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/forum")
            .route("/api/threads", actix_web::web::post().to(handler::create_thread))
            .route(
                "/api/threads/{id}",
                actix_web::web::put().to(handler::edit_thread),
            )
            .route(
                "/api/threads/{id}",
                actix_web::web::delete().to(handler::delete_thread),
            )
            .route(
                "/api/threads/{id}/replies",
                actix_web::web::post().to(handler::create_reply),
            )
            .route(
                "/api/replies/{id}",
                actix_web::web::delete().to(handler::delete_reply),
            ),
    );
}