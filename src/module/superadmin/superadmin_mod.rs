use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;
use crate::web::data::String as Str;

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserRow {
    pub id: u32,
    pub username: Str,
    pub fullname: Str,
    pub last_login: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct FileRow {
    pub id: u32,
    pub filename: Str,
    pub original_name: Str,
    pub mime_type: Str,
    pub file_size: u64,
    pub uploaded_by: u32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct DbQueryInput {
    pub sql: Str,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub search: Option<Str>,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(FromRow, Debug, Serialize)]
pub struct ActivityLog {
    pub id: u32,
    pub user_id: Option<u32>,
    pub action: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct EditUserInput {
    pub fullname: Str,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordInput {
    pub new_password: Str,
    pub confirm_password: Str,
}