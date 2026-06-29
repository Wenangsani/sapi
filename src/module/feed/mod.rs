pub mod handler;

use actix_web::web::ServiceConfig;
use actix_web::web;

/// Route publik (akses halaman feed — akan dicek auth di handler)
pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/feed")
            .route("", web::get().to(handler::page_feed))
    );
}

/// Route yang membutuhkan autentikasi (API endpoints)
pub fn gate_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/feed")
            // --- Post CRUD ---
            .route("/posts", web::get().to(handler::get_posts))
            .route("/posts", web::post().to(handler::create_post))
            .route("/posts/{post_id}", web::get().to(handler::get_post_detail))
            .route("/posts/{post_id}", web::put().to(handler::update_post))
            .route("/posts/{post_id}", web::delete().to(handler::delete_post))
            // --- Like Post ---
            .route("/posts/{post_id}/like", web::post().to(handler::toggle_like_post))
            // --- Save Post ---
            .route("/posts/{post_id}/save", web::post().to(handler::toggle_save_post))
            // --- Report Post ---
            .route("/posts/{post_id}/report", web::post().to(handler::report_post))
            // --- Comments ---
            .route("/posts/{post_id}/comments", web::get().to(handler::get_comments))
            .route("/posts/{post_id}/comments", web::post().to(handler::create_comment))
            .route("/comments/{comment_id}", web::put().to(handler::update_comment))
            .route("/comments/{comment_id}", web::delete().to(handler::delete_comment))
            .route("/comments/{comment_id}/like", web::post().to(handler::toggle_like_comment))
            // --- Notifikasi ---
            .route("/notifications", web::get().to(handler::get_notifications))
            .route("/notifications/read", web::post().to(handler::mark_all_read))
            .route("/notifications/{notif_id}/read", web::post().to(handler::mark_read))
            // --- Saved Posts ---
            .route("/saved", web::get().to(handler::get_saved_posts))
    );
}
