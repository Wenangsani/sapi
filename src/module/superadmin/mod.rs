pub mod handler;

use actix_web::web::ServiceConfig;
use actix_web::web::{self, scope};

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/superadmin")
            .route("", web::get().to(handler::page_dashboard))
            .route("/users", web::get().to(handler::page_users))
            .route("/files", web::get().to(handler::page_files))
            .route("/database", web::get().to(handler::page_database))
            .route("/logs", web::get().to(handler::page_logs)),
    );
}

pub fn gate_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/superadmin")
            // Users API
            .route("/users", web::get().to(handler::api_list_users))
            .route("/users/{id}", web::delete().to(handler::api_delete_user))
            // Files API
            .route("/files", web::get().to(handler::api_list_files))
            .route("/files/upload", web::post().to(handler::api_upload_file))
            .route("/files/{id}", web::delete().to(handler::api_delete_file))
            // Database API
            .route("/database", web::post().to(handler::api_db_query)) // konsol SQL
            // Database Explorer APIs
            .route("/database/tables", web::get().to(handler::api_list_tables))
            .route("/database/tables/{table}/columns", web::get().to(handler::api_table_columns))
            .route("/database/tables/{table}/rows", web::get().to(handler::api_table_rows))
            .route("/database/tables/{table}/rows", web::post().to(handler::api_insert_row))
            .route("/database/tables/{table}/rows/{id}", web::put().to(handler::api_update_row))
            .route("/database/tables/{table}/rows/{id}", web::delete().to(handler::api_delete_row))
            // Stats API
            .route("/stats", web::get().to(handler::api_stats))
            .route("/logs", web::get().to(handler::api_get_logs)),
    );
}