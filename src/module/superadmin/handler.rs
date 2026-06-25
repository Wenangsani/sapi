use crate::web::{Pool, Session, Response, ApiResponse};
use crate::web::from::{Path, Json};
use crate::web::data::String as Str;
use actix_multipart::Multipart as ActixMultipart;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;
use tokio::fs;
use sqlx::{Column, Row};

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserRow {
    pub id: u32,
    pub username: Str,
    pub fullname: Str,
    pub last_login: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct FileRow {
    pub id: u32,
    pub filename: Str,
    pub original_name: Str,
    pub mime_type: Str,
    pub file_size: u64,
    pub uploaded_by: u32,
    pub created_at: chrono::NaiveDateTime,
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

// ── Pages (HTML serve) ────────────────────────────────────────────────────────

pub async fn page_dashboard(session: Session) -> Response {
    let _user_id = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_dashboard.html"))
}

pub async fn page_users(session: Session) -> Response {
    let _user_id = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_users.html"))
}

pub async fn page_files(session: Session) -> Response {
    let _user_id = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_files.html"))
}

pub async fn page_database(session: Session) -> Response {
    let _user_id = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_database.html"))
}

// ── API: Stats ────────────────────────────────────────────────────────────────

pub async fn api_stats(session: Session, pool: Pool) -> Response {
    let _user_id = auth!(session);

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
    query: actix_web::web::Query<PaginationQuery>,
) -> Response {
    let _user_id = auth!(session);

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
    // Fix 1: auth! mengembalikan Option<u32>, unwrap dengan match
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
    query: actix_web::web::Query<PaginationQuery>,
) -> Response {
    let _user_id = auth!(session);

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
    mut payload: ActixMultipart,
) -> Response {
    // Fix 1 (sama): auth! return Option<u32>
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

        // Fix 2: content_disposition() return Option<&ContentDisposition>
        // gunakan .as_ref().and_then() untuk memanggil get_filename()
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
            uuid::Uuid::new_v4()
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
    let _user_id = auth!(session);
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

// ── API: Database Query ───────────────────────────────────────────────────────

pub async fn api_db_query(
    session: Session,
    pool: Pool,
    body: Json<DbQueryInput>,
) -> Response {
    let _user_id = auth!(session);

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

    // SESUDAH — leak ke 'static agar memenuhi bound SqlSafeStr
    let sql: &'static str = Box::leak(body.sql.clone().into_boxed_str());

    // Fix 4: trait Column & Row di-import di atas agar col.name() & row.columns() tersedia
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