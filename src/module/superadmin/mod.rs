pub mod dashboard;
pub mod users;
pub mod files;
pub mod database;
pub mod logs;
pub mod security;
pub mod superadmin_mod;

use actix_web::web::ServiceConfig;
use actix_web::web::{self, scope};

pub fn open_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/superadmin")
            .route("", web::get().to(dashboard::page_dashboard))
            .route("/users", web::get().to(users::page_users))
            .route("/files", web::get().to(files::page_files))
            .route("/database", web::get().to(database::page_database))
            .route("/logs", web::get().to(logs::page_logs)),
    );
}

pub fn gate_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/superadmin")
            // Users API
            .route("/users", web::get().to(users::api_list_users))
            .route("/users/{id}", web::put().to(users::api_edit_user))
            .route("/users/{id}", web::delete().to(users::api_delete_user))
            .route("/users/{id}/password", web::put().to(users::api_change_password))
            // Files API
            .route("/files", web::get().to(files::api_list_files))
            .route("/files/upload", web::post().to(files::api_upload_file))
            .route("/files/{id}", web::delete().to(files::api_delete_file))
            // Database API
            .route("/database", web::post().to(database::api_db_query))
            // Database Explorer APIs
            .route("/database/tables", web::get().to(database::api_list_tables))
            .route("/database/tables/{table}/columns", web::get().to(database::api_table_columns))
            .route("/database/tables/{table}/rows", web::get().to(database::api_table_rows))
            .route("/database/tables/{table}/rows", web::post().to(database::api_insert_row))
            .route("/database/tables/{table}/rows/{id}", web::put().to(database::api_update_row))
            .route("/database/tables/{table}/rows/{id}", web::delete().to(database::api_delete_row))
            // Stats API
            .route("/stats", web::get().to(dashboard::api_stats))
            .route("/logs", web::get().to(logs::api_get_logs)),
    );
}