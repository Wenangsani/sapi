use crate::web::{Pool, Session, Request, Response, ApiResponse};
use crate::web::from::{Path, Json};
use crate::web::data::String as Str;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use tokio::fs;
use sqlx::{Column, Row, FromRow};
use chrono::NaiveDateTime;
use actix_web::web::Query;
use uuid::Uuid;
use std::collections::HashMap;

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

// ── Pages (HTML serve) ────────────────────────────────────────────────────────

pub async fn page_dashboard(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_dashboard.html"))
}

pub async fn page_users(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_users.html"))
}

pub async fn page_files(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_files.html"))
}

pub async fn page_database(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_database.html"))
}

pub async fn page_logs(session: Session) -> Response {
    if auth!(session).is_none() {
        return Response::Found()
            .append_header(("Location", "/auth/login"))
            .finish();
    }
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_logs.html"))
}

pub async fn page_security(session: Session) -> Response {
    if auth!(session).is_none() {
        return Response::Found()
            .append_header(("Location", "/auth/login"))
            .finish();
    }
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_security.html"))
}

// ── API: Stats ────────────────────────────────────────────────────────────────

pub async fn api_stats(session: Session, pool: Pool) -> Response {
    let _ = auth!(session);

    let user_count: (i64,) = match sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("DB error: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let file_count: (i64,) = match sqlx::query_as("SELECT COUNT(*) FROM uploaded_files")
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("DB error: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total_size: (Option<i64>,) =
        match sqlx::query_as("SELECT CAST(SUM(file_size) AS SIGNED) FROM uploaded_files")
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("DB error: {}", e),
                    data: None,
                    meta: None,
                });
            }
        };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!({
            "user_count": user_count.0,
            "file_count": file_count.0,
            "total_storage_bytes": total_size.0.unwrap_or(0),
        })),
        meta: None,
    })
}

// ── API: Users ────────────────────────────────────────────────────────────────

pub async fn api_list_users(
    session: Session,
    pool: Pool,
    query: Query<PaginationQuery>,
) -> Response {
    let _ = auth!(session);

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let search_pattern = query
        .search
        .as_deref()
        .map(|s| format!("%{}%", s))
        .unwrap_or_else(|| "%".into());

    let rows: Vec<UserRow> = match sqlx::query_as!(
        UserRow,
        r#"SELECT id, username, fullname, last_login, created_at
           FROM users
           WHERE username LIKE ? OR fullname LIKE ?
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#,
        search_pattern,
        search_pattern,
        limit,
        offset
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("DB error: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total: (i64,) = match sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE username LIKE ? OR fullname LIKE ?",
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("DB error: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total_pages = ((total.0 as f64) / (limit as f64)).ceil() as i64;

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!(rows)),
        meta: Some(json!({
            "page": page,
            "limit": limit,
            "total": total.0,
            "total_pages": total_pages,
        })),
    })
}

pub async fn api_delete_user(
    session: Session,
    pool: Pool,
    path: Path<(u32,)>,
) -> Response {
    let user_id = match auth!(session) {
        Some(id) => id,
        None => {
            return Response::Unauthorized().json(ApiResponse {
                success: false,
                message: "Tidak terautentikasi".into(),
                data: None,
                meta: None,
            });
        }
    };

    let target_id = path.into_inner().0;

    if target_id == user_id {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Tidak dapat menghapus akun sendiri".into(),
            data: None,
            meta: None,
        });
    }

    match sqlx::query!("DELETE FROM users WHERE id = ?", target_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(r) if r.rows_affected() == 0 => Response::NotFound().json(ApiResponse {
            success: false,
            message: "User tidak ditemukan".into(),
            data: None,
            meta: None,
        }),
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "User berhasil dihapus".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("DB error: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ── API: Files ────────────────────────────────────────────────────────────────

pub async fn api_list_files(
    session: Session,
    pool: Pool,
    query: Query<PaginationQuery>,
) -> Response {
    let _ = auth!(session);

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let rows: Vec<FileRow> = match sqlx::query_as!(
        FileRow,
        r#"SELECT id, filename, original_name, mime_type, file_size, uploaded_by, created_at
           FROM uploaded_files
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#,
        limit,
        offset
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("DB error: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total: (i64,) =
        match sqlx::query_as("SELECT COUNT(*) FROM uploaded_files")
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("DB error: {}", e),
                    data: None,
                    meta: None,
                });
            }
        };

    let total_pages = ((total.0 as f64) / (limit as f64)).ceil() as i64;

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!(rows)),
        meta: Some(json!({
            "page": page,
            "limit": limit,
            "total": total.0,
            "total_pages": total_pages,
        })),
    })
}

pub async fn api_upload_file(
    session: Session,
    pool: Pool,
    mut payload: Multipart,
) -> Response {
    let user_id = match auth!(session) {
        Some(id) => id,
        None => {
            return Response::Unauthorized().json(ApiResponse {
                success: false,
                message: "Tidak terautentikasi".into(),
                data: None,
                meta: None,
            });
        }
    };

    let upload_dir = "uploads";
    if let Err(e) = fs::create_dir_all(upload_dir).await {
        return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal membuat direktori upload: {}", e),
            data: None,
            meta: None,
        });
    }

    let mut saved_files: Vec<serde_json::Value> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                return Response::BadRequest().json(ApiResponse {
                    success: false,
                    message: format!("Error membaca upload: {}", e),
                    data: None,
                    meta: None,
                });
            }
        };

        let original_name = field
            .content_disposition()
            .as_ref()
            .and_then(|cd| cd.get_filename())
            .unwrap_or("unknown")
            .to_string();

        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let unique_name = format!(
            "{}_{}.{}",
            chrono::Utc::now().timestamp_millis(),
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("x"),
            ext
        );

        let mime = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".into());

        let file_path = format!("{}/{}", upload_dir, unique_name);
        let mut file = match std::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal membuat file: {}", e),
                    data: None,
                    meta: None,
                });
            }
        };

        let mut total_size: u64 = 0;
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(e) => {
                    return Response::BadRequest().json(ApiResponse {
                        success: false,
                        message: format!("Error baca chunk: {}", e),
                        data: None,
                        meta: None,
                    });
                }
            };
            total_size += data.len() as u64;
            if let Err(e) = file.write_all(&data) {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal menulis file: {}", e),
                    data: None,
                    meta: None,
                });
            }
        }

        match sqlx::query!(
            r#"INSERT INTO uploaded_files (filename, original_name, mime_type, file_size, uploaded_by)
               VALUES (?, ?, ?, ?, ?)"#,
            unique_name,
            original_name,
            mime,
            total_size,
            user_id
        )
        .execute(pool.get_ref())
        .await
        {
            Ok(r) => {
                saved_files.push(json!({
                    "id": r.last_insert_id(),
                    "filename": unique_name,
                    "original_name": original_name,
                    "file_size": total_size,
                }));
            }
            Err(e) => {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("DB error: {}", e),
                    data: None,
                    meta: None,
                });
            }
        }
    }

    Response::Ok().json(ApiResponse {
        success: true,
        message: format!("{} file berhasil diupload", saved_files.len()),
        data: Some(json!(saved_files)),
        meta: None,
    })
}

pub async fn api_delete_file(
    session: Session,
    pool: Pool,
    path: Path<(u32,)>,
) -> Response {
    let _ = auth!(session);
    let file_id = path.into_inner().0;

    let row = match sqlx::query!(
        "SELECT filename FROM uploaded_files WHERE id = ?",
        file_id
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "File tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("DB error: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let file_path = format!("uploads/{}", row.filename);
    let _ = fs::remove_file(&file_path).await;

    match sqlx::query!("DELETE FROM uploaded_files WHERE id = ?", file_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "File berhasil dihapus".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("DB error: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ── API: Database Console ─────────────────────────────────────────────────────

pub async fn api_db_query(
    session: Session,
    pool: Pool,
    body: Json<DbQueryInput>,
) -> Response {
    let _ = auth!(session);

    let sql_lower = body.sql.trim().to_lowercase();

    if !sql_lower.starts_with("select")
        && !sql_lower.starts_with("show")
        && !sql_lower.starts_with("describe")
    {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Hanya perintah SELECT, SHOW, dan DESCRIBE yang diizinkan".into(),
            data: None,
            meta: None,
        });
    }

    let sql: &'static str = Box::leak(body.sql.clone().into_boxed_str());

    match sqlx::query(sql)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => {
            let result: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let mut map = serde_json::Map::new();
                    for (i, col) in row.columns().iter().enumerate() {
                        let val = if let Ok(v) = row.try_get::<Option<Str>, _>(i) {
                            v.map(|s| json!(s)).unwrap_or(json!(null))
                        } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                            v.map(|n| json!(n)).unwrap_or(json!(null))
                        } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                            v.map(|f| json!(f)).unwrap_or(json!(null))
                        } else {
                            json!(null)
                        };
                        map.insert(col.name().to_string(), val);
                    }
                    serde_json::Value::Object(map)
                })
                .collect();

            Response::Ok().json(ApiResponse {
                success: true,
                message: format!("{} baris ditemukan", result.len()),
                data: Some(json!(result)),
                meta: None,
            })
        }
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Query error: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ── API: Database Explorer ────────────────────────────────────────────────────

/// GET /gate/superadmin/database/tables
pub async fn api_list_tables(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let sql: &'static str = "SHOW TABLES";
    match sqlx::query(sql)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => {
            let tables: Vec<String> = rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!({ "tables": tables })),
                meta: None,
            })
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mengambil tabel: {}", e),
            data: None,
            meta: None,
        }),
    }
}

/// GET /gate/superadmin/database/tables/{table}/columns
pub async fn api_table_columns(
    session: Session,
    pool: Pool,
    path: Path<(String,)>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let table = path.into_inner().0;
    let sql: &'static str = Box::leak(format!("DESCRIBE `{}`", table).into_boxed_str());

    match sqlx::query(sql)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => {
            let columns: Vec<Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "name": row.get::<String, _>(0),
                        "type": row.get::<String, _>(1),
                        "null": row.get::<String, _>(2),
                        "key": row.get::<String, _>(3),
                        "default": row.get::<Option<String>, _>(4),
                        "extra": row.get::<String, _>(5)
                    })
                })
                .collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!({ "columns": columns })),
                meta: None,
            })
        }
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal deskripsi tabel: {}", e),
            data: None,
            meta: None,
        }),
    }
}

/// GET /gate/superadmin/database/tables/{table}/rows?page=...&limit=...
pub async fn api_table_rows(
    session: Session,
    pool: Pool,
    path: Path<(String,)>,
    query: Query<PaginationQuery>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let table = path.into_inner().0;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    // Hitung total
    let count_sql: &'static str = Box::leak(format!("SELECT COUNT(*) FROM `{}`", table).into_boxed_str());
    let total: i64 = match sqlx::query_scalar(count_sql)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(c) => c,
        Err(e) => {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: format!("Gagal menghitung baris: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    // Ambil data
    let data_sql: &'static str = Box::leak(
        format!("SELECT * FROM `{}` LIMIT {} OFFSET {}", table, limit, offset).into_boxed_str(),
    );

    let rows: Vec<Value> = match sqlx::query(data_sql)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(rows) => rows.iter().map(|row| {
            let mut map = serde_json::Map::new();
            for (i, col) in row.columns().iter().enumerate() {
                let val: Value = try_get_value(row, i);
                map.insert(col.name().to_string(), val);
            }
            Value::Object(map)
        }).collect(),
        Err(e) => {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: format!("Gagal mengambil data: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!({
            "rows": rows,
            "page": page,
            "total_pages": total_pages,
            "total": total,
        })),
        meta: None,
    })
}

/// POST /gate/superadmin/database/tables/{table}/rows
pub async fn api_insert_row(
    session: Session,
    pool: Pool,
    path: Path<(String,)>,
    body: Json<HashMap<String, Value>>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let table = path.into_inner().0;
    let data = body.into_inner();

    if data.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Data kosong".into(),
            data: None,
            meta: None,
        });
    }

    let columns: Vec<&String> = data.keys().collect();
    let values: Vec<String> = columns.iter().map(|k| format!("'{}'", data[*k].as_str().unwrap_or(""))).collect();

    let sql: &'static str = Box::leak(format!(
        "INSERT INTO `{}` ({}) VALUES ({})",
        table,
        columns.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", "),
        values.join(", ")
    ).into_boxed_str());

    match sqlx::query(sql).execute(pool.get_ref()).await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Baris berhasil ditambahkan".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal insert: {}", e),
            data: None,
            meta: None,
        }),
    }
}

/// PUT /gate/superadmin/database/tables/{table}/rows/{id}
pub async fn api_update_row(
    session: Session,
    pool: Pool,
    path: Path<(String, String)>,
    body: Json<HashMap<String, Value>>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let (table, id) = path.into_inner();
    let data = body.into_inner();

    if data.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Data kosong".into(),
            data: None,
            meta: None,
        });
    }

    let set_clause = data.iter()
        .map(|(k, v)| format!("`{}` = '{}'", k, v.as_str().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(", ");

    let sql: &'static str = Box::leak(
        format!("UPDATE `{}` SET {} WHERE id = '{}'", table, set_clause, id).into_boxed_str()
    );

    match sqlx::query(sql).execute(pool.get_ref()).await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Baris berhasil diperbarui".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal update: {}", e),
            data: None,
            meta: None,
        }),
    }
}

/// DELETE /gate/superadmin/database/tables/{table}/rows/{id}
pub async fn api_delete_row(
    session: Session,
    pool: Pool,
    path: Path<(String, String)>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let (table, id) = path.into_inner();

    let sql: &'static str = Box::leak(
        format!("DELETE FROM `{}` WHERE id = '{}'", table, id).into_boxed_str()
    );

    match sqlx::query(sql).execute(pool.get_ref()).await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Baris berhasil dihapus".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal delete: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// Helper untuk mengkonversi nilai kolom ke JSON
fn try_get_value(row: &sqlx::mysql::MySqlRow, index: usize) -> Value {
    if let Ok(v) = row.try_get::<Option<String>, _>(index) {
        return v.map(|s| json!(s)).unwrap_or(json!(null));
    }
    if let Ok(v) = row.try_get::<Option<i64>, _>(index) {
        return v.map(|n| json!(n)).unwrap_or(json!(null));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(index) {
        return v.map(|f| json!(f)).unwrap_or(json!(null));
    }
    json!(null)
}

// ── API: Logs ─────────────────────────────────────────────────────────────────

/// GET /gate/superadmin/logs?page=...&limit=...
pub async fn api_get_logs(
    session: Session,
    pool: Pool,
    query: Query<LogsQuery>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = match sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM activity_logs"
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(c) => c,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal menghitung log: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let logs: Vec<ActivityLog> = match sqlx::query_as!(
        ActivityLog,
        r#"SELECT id, user_id, action, details, ip_address, created_at 
           FROM activity_logs 
           ORDER BY created_at DESC 
           LIMIT ? OFFSET ?"#,
        limit as u32,
        offset as u32
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(logs) => logs,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal mengambil log: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Log berhasil diambil".into(),
        data: Some(serde_json::json!({
            "logs": logs,
            "total": total,
            "page": page,
            "total_pages": total_pages
        })),
        meta: None,
    })
}

// ── API: Security ─────────────────────────────────────────────────────────────

/// GET /gate/superadmin/security
pub async fn api_get_security_stats(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    // Contoh statistik keamanan (dapat disesuaikan)
    let weak_password_count: i64 = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE LENGTH(password) < 8"
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(c) => c,
        Err(_) => 0,
    };

    let two_fa_count: i64 = 0; // implementasi sesuai kebutuhan

    let active_sessions: i64 = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE last_login > NOW() - INTERVAL 1 DAY"
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(c) => c,
        Err(_) => 0,
    };

    let failed_login_attempts: i64 = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM activity_logs WHERE action = 'login_failed' AND created_at > NOW() - INTERVAL 1 DAY"
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(c) => c,
        Err(_) => 0,
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!({
            "weak_password_count": weak_password_count,
            "two_fa_count": two_fa_count,
            "active_sessions": active_sessions,
            "failed_login_attempts": failed_login_attempts,
        })),
        meta: None,
    })
}

/// POST /gate/superadmin/security/clear-sessions
pub async fn api_clear_expired_sessions(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    match sqlx::query!("DELETE FROM users WHERE last_login < NOW() - INTERVAL 1 DAY")
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Sesi kedaluwarsa berhasil dibersihkan".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal membersihkan sesi: {}", e),
            data: None,
            meta: None,
        }),
    }
}

/// POST /gate/superadmin/security/reset-failed
pub async fn api_reset_failed_attempts(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    match sqlx::query!("DELETE FROM activity_logs WHERE action = 'login_failed'")
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Riwayat percobaan login gagal telah direset".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mereset: {}", e),
            data: None,
            meta: None,
        }),
    }
}

/// POST /gate/superadmin/security/toggle-lock
pub async fn api_toggle_system_lock(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    // Implementasi toggle system lock (contoh: ubah flag di tabel settings)
    // Untuk sementara kembalikan sukses
    Response::Ok().json(ApiResponse {
        success: true,
        message: "Status kunci sistem telah diubah".into(),
        data: None,
        meta: None,
    })
}