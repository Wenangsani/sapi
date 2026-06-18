pub mod handler;

use actix_web::web::ServiceConfig;
use actix_web::web::{get, post};

/// Route tanpa proteksi gate (halaman publik + endpoint yang otentikasinya
/// dicek manual via auth!() di dalam handler, contoh: create thread, reply, unlock password)
pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/forum")
            // Halaman
            .route("", get().to(handler::page_list))
            .route("/{id}", get().to(handler::page_detail))
            // Data thread list (sidebar tag, list thread, search, pagination)
            .route("/api/tags", get().to(handler::list_tags))
            .route("/api/threads", get().to(handler::list_threads))
            .route("/api/threads", post().to(handler::create_thread))
            // Detail thread + reply
            .route("/api/threads/{id}", get().to(handler::get_thread_detail))
            .route("/api/threads/{id}/unlock", post().to(handler::unlock_thread))
            .route("/api/threads/{id}/replies", get().to(handler::list_replies))
            .route("/api/threads/{id}/replies", post().to(handler::create_reply)),
    );
}

/// Route dengan proteksi gate (khusus API, prefix /gate sudah ditambahkan di main.rs)
pub fn gate_routes(_cfg: &mut ServiceConfig) {
    // Modul forum bersifat Public, tidak ada endpoint khusus gate.
}