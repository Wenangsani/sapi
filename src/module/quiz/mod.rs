use actix_web::web::ServiceConfig;

pub mod handler;

pub fn open_routes(cfg: &mut ServiceConfig) {
    use actix_web::web::{scope, get, post};
    use handler::*;

    cfg.service(
        scope("/quiz")
            // ── Halaman HTML ──────────────────────────────────
            // Halaman list quiz (public)
            .route("", get().to(page_list))
            // Halaman buat quiz — HARUS sebelum /{quiz_id} agar "/create" tidak ditangkap sebagai quiz_id
            .route("/create", get().to(page_create))
            // Halaman info detail quiz (public)
            .route("/{quiz_id}", get().to(page_info))
            // Halaman kerjakan quiz (auth dicek di handler)
            .route("/{quiz_id}/take", get().to(page_take))

            // ── API Data ──────────────────────────────────────
            // API: daftar semua quiz published
            .route("/api/list", get().to(api_list_quiz))
            // API: info satu quiz
            .route("/api/{quiz_id}/info", get().to(api_quiz_info))
            // API: soal-soal quiz (butuh login)
            .route("/{quiz_id}/questions", get().to(api_get_questions))
            // API: leaderboard
            .route("/{quiz_id}/leaderboard", get().to(api_leaderboard))
            // API: submit jawaban
            .route("/{quiz_id}/submit", post().to(api_submit))
            // API: buat quiz baru (butuh login)
            .route("/create", post().to(api_create_quiz)),
    );
}

pub fn gate_routes(cfg: &mut ServiceConfig) {
    use actix_web::web::{scope, delete, put};
    use handler::*;

    cfg.service(
        scope("/quiz")
            .route("/{quiz_id}", delete().to(api_delete_quiz))
            .route("/{quiz_id}/publish", put().to(api_toggle_publish)),
    );
}