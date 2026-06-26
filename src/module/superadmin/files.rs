use crate::web::{Pool, Session, Response, ApiResponse};
use crate::web::from::Path;
use actix_multipart::Multipart;
use actix_web::web::Query;
use futures_util::StreamExt;
use serde_json::json;
use std::io::Write;
use tokio::fs;
use uuid::Uuid;
use crate::module::superadmin::superadmin_mod::{FileRow, PaginationQuery};

pub async fn page_files(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_files.html"))
}

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
