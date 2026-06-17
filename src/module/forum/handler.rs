use crate::web::{Pool, Session, Cookie, Request, Response, ApiResponse};
use crate::web::from::{Data, Path, Json, Form};
use crate::web::data::{Int, UInt, String, Date};
use actix_web::web::Query;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============================================================
// Struct — Query & Path
// ============================================================

#[derive(Deserialize)]
pub struct ThreadQuery {
    pub tag: Option<String>,
    pub page: Option<Int>,
    pub per_page: Option<Int>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct PathId {
    pub id: Int,
}

// ============================================================
// Struct — Request Body
// ============================================================

#[derive(Deserialize)]
pub struct CreateThreadForm {
    pub title: String,
    pub content: String,
    pub tag_id: Option<Int>,
    pub access_type: String,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct EditThreadForm {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tag_id: Option<Int>,
    pub access_type: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct ReplyForm {
    pub content: String,
}

#[derive(Deserialize)]
pub struct VerifyPasswordForm {
    pub password: String,
}

// ============================================================
// Struct — Row (SQLx FromRow)
// ============================================================

#[derive(sqlx::FromRow)]
pub struct ThreadListRow {
    pub id: Int,
    pub title: String,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub tag_slug: Option<String>,
    pub author_name: Option<String>,
    pub reply_count: i64,
    pub access_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct ThreadDetailRow {
    pub id: Int,
    pub title: String,
    pub content: String,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub tag_slug: Option<String>,
    pub author_id: Int,
    pub author_name: Option<String>,
    pub access_type: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Row khusus untuk operasi yang butuh password_hash dan author_id
#[derive(sqlx::FromRow)]
pub struct ThreadAuthRow {
    pub id: Int,
    pub author_id: Int,
    pub access_type: String,
    pub password_hash: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct ReplyRow {
    pub id: Int,
    pub content: String,
    pub author_id: Int,
    pub author_name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct TagRow {
    pub id: Int,
    pub name: String,
    pub slug: String,
    pub color: String,
}

#[derive(sqlx::FromRow)]
pub struct CountRow {
    pub total: i64,
}

// ============================================================
// Helper — format NaiveDateTime ke RFC3339 string
// ============================================================

fn fmt_dt(dt: &NaiveDateTime) -> String {
    dt.and_utc().to_rfc3339()
}

// ============================================================
// HALAMAN HTML
// ============================================================

pub async fn page_forum() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_forum.html"))
}

pub async fn page_thread() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_thread.html"))
}

// ============================================================
// API — List Threads (publik)
// ============================================================

pub async fn list_threads(pool: Pool, query: Query<ThreadQuery>) -> Response {
    let tag = query.tag.clone().unwrap_or_default();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(15).max(1).min(50);
    let offset = (page - 1) * per_page;
    let search = query.search.clone().unwrap_or_default();
    let search_pattern = format!("%{}%", search);

    // Hitung total
    let count_result = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) AS total
         FROM forum_threads t
         LEFT JOIN forum_tags tg ON t.tag_id = tg.id
         WHERE (tg.slug = ? OR ? = '')
           AND (t.title LIKE ? OR ? = '')",
    )
    .bind(&tag)
    .bind(&tag)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_one(pool.get_ref())
    .await;

    let total = match count_result {
        Ok(row) => row.total as Int,
        Err(_) => 0,
    };

    let total_pages: Int = if total > 0 {
        ((total as f64) / (per_page as f64)).ceil() as Int
    } else {
        0
    };

    // Ambil data
    let result = sqlx::query_as::<_, ThreadListRow>(
        "SELECT t.id, t.title,
                tg.name AS tag_name, tg.color AS tag_color, tg.slug AS tag_slug,
                u.name AS author_name,
                (SELECT COUNT(*) FROM forum_replies r WHERE r.thread_id = t.id) AS reply_count,
                t.access_type, t.created_at
         FROM forum_threads t
         LEFT JOIN forum_tags tg ON t.tag_id = tg.id
         LEFT JOIN users u ON t.user_id = u.id
         WHERE (tg.slug = ? OR ? = '')
           AND (t.title LIKE ? OR ? = '')
         ORDER BY t.created_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(&tag)
    .bind(&tag)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    let threads = match result {
        Ok(t) => t,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "Gagal memuat thread".into(),
                data: None,
                meta: None,
            });
        }
    };

    let thread_json: Vec<serde_json::Value> = threads
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "title": t.title,
                "tag_name": t.tag_name,
                "tag_color": t.tag_color,
                "tag_slug": t.tag_slug,
                "author_name": t.author_name,
                "reply_count": t.reply_count,
                "access_type": t.access_type,
                "created_at": fmt_dt(&t.created_at),
            })
        })
        .collect();

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Daftar thread berhasil dimuat".into(),
        data: Some(json!(thread_json)),
        meta: Some(json!({
            "page": page,
            "per_page": per_page,
            "total": total,
            "total_pages": total_pages,
        })),
    })
}

// ============================================================
// API — Get Thread Detail (publik, akses dicek di handler)
// ============================================================

pub async fn get_thread(pool: Pool, session: Session, path: Path<PathId>) -> Response {
    let thread_id = path.id;

    let thread = match sqlx::query_as::<_, ThreadDetailRow>(
        "SELECT t.id, t.title, t.content,
                tg.name AS tag_name, tg.color AS tag_color, tg.slug AS tag_slug,
                t.user_id AS author_id, u.name AS author_name,
                t.access_type, t.created_at, t.updated_at
         FROM forum_threads t
         LEFT JOIN forum_tags tg ON t.tag_id = tg.id
         LEFT JOIN users u ON t.user_id = u.id
         WHERE t.id = ?",
    )
    .bind(thread_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(t) => t,
        Err(_) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Thread tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
    };

    // Cek akses berdasarkan access_type
    match thread.access_type.as_str() {
        "user" => {
            // session.get() bersifat sinkron, TIDAK perlu .await
            let uid: Option<Int> = match session.get("user_id") {
                Ok(v) => v,
                Err(_) => None,
            };
            if uid.is_none() {
                return Response::Unauthorized().json(ApiResponse {
                    success: false,
                    message: "login_required".into(),
                    data: None,
                    meta: None,
                });
            }
        }
        "password" => {
            let key = format!("forum_pw_{}", thread_id);
            // session.get() bersifat sinkron, TIDAK perlu .await
            let verified: Option<bool> = match session.get(&key) {
                Ok(v) => v,
                Err(_) => None,
            };
            if verified != Some(true) {
                return Response::Ok().json(ApiResponse {
                    success: false,
                    message: "password_required".into(),
                    data: Some(json!({
                        "thread_id": thread_id,
                        "title": thread.title,
                    })),
                    meta: None,
                });
            }
        }
        _ => {} // public — lanjut
    }

    // Ambil balasan
    let replies = match sqlx::query_as::<_, ReplyRow>(
        "SELECT r.id, r.content, r.user_id AS author_id, u.name AS author_name, r.created_at
         FROM forum_replies r
         LEFT JOIN users u ON r.user_id = u.id
         WHERE r.thread_id = ?
         ORDER BY r.created_at ASC",
    )
    .bind(thread_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "Gagal memuat balasan".into(),
                data: None,
                meta: None,
            });
        }
    };

    let reply_count = replies.len() as Int;

    let replies_json: Vec<serde_json::Value> = replies
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "content": r.content,
                "author_id": r.author_id,
                "author_name": r.author_name,
                "created_at": fmt_dt(&r.created_at),
            })
        })
        .collect();

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Thread berhasil dimuat".into(),
        data: Some(json!({
            "thread": {
                "id": thread.id,
                "title": thread.title,
                "content": thread.content,
                "tag_name": thread.tag_name,
                "tag_color": thread.tag_color,
                "tag_slug": thread.tag_slug,
                "author_id": thread.author_id,
                "author_name": thread.author_name,
                "access_type": thread.access_type,
                "reply_count": reply_count,
                "created_at": fmt_dt(&thread.created_at),
                "updated_at": fmt_dt(&thread.updated_at),
            },
            "replies": replies_json,
        })),
        meta: None,
    })
}

// ============================================================
// API — List Tags (publik)
// ============================================================

pub async fn list_tags(pool: Pool) -> Response {
    let tags = match sqlx::query_as::<_, TagRow>(
        "SELECT id, name, slug, color FROM forum_tags ORDER BY name ASC",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(t) => t,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "Gagal memuat tag".into(),
                data: None,
                meta: None,
            });
        }
    };

    let tags_json: Vec<serde_json::Value> = tags
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "slug": t.slug,
                "color": t.color,
            })
        })
        .collect();

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Tag berhasil dimuat".into(),
        data: Some(json!(tags_json)),
        meta: None,
    })
}

// ============================================================
// API — Verify Password (publik)
// ============================================================

pub async fn verify_password(
    pool: Pool,
    session: Session,
    path: Path<PathId>,
    form: Json<VerifyPasswordForm>,
) -> Response {
    let thread_id = path.id;

    // Gunakan ThreadAuthRow yang punya field password_hash
    let thread = match sqlx::query_as::<_, ThreadAuthRow>(
        "SELECT id, user_id AS author_id, access_type, password_hash
         FROM forum_threads
         WHERE id = ?",
    )
    .bind(thread_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(t) => t,
        Err(_) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Thread tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
    };

    if thread.access_type != "password" {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Thread ini tidak dilindungi password".into(),
            data: None,
            meta: None,
        });
    }

    let stored_hash = match &thread.password_hash {
        Some(h) => h,
        None => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "Thread belum memiliki password".into(),
                data: None,
                meta: None,
            });
        }
    };

    match verify(&form.password, stored_hash) {
        Ok(true) => {
            let key = format!("forum_pw_{}", thread_id);
            // session.insert() bersifat sinkron, TIDAK perlu .await
            let _ = session.insert(&key, true);
            Response::Ok().json(ApiResponse {
                success: true,
                message: "Password benar".into(),
                data: Some(json!({ "thread_id": thread_id })),
                meta: None,
            })
        }
        Ok(false) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Password salah".into(),
            data: None,
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "Gagal memverifikasi password".into(),
            data: None,
            meta: None,
        }),
    }
}

// ============================================================
// API — Create Thread (login required)
// ============================================================

pub async fn create_thread(pool: Pool, session: Session, form: Json<CreateThreadForm>) -> Response {
    let user_id = auth!(session);

    let title = form.title.trim().to_string();
    let content = form.content.trim().to_string();

    if title.is_empty() || content.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Judul dan konten wajib diisi".into(),
            data: None,
            meta: None,
        });
    }

    let access_type = form.access_type.trim().to_string();
    if access_type != "public" && access_type != "user" && access_type != "password" {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Tipe akses tidak valid".into(),
            data: None,
            meta: None,
        });
    }

    let password_hash = if access_type == "password" {
        let pwd = match &form.password {
            Some(p) if !p.is_empty() => p.clone(),
            _ => {
                return Response::BadRequest().json(ApiResponse {
                    success: false,
                    message: "Password wajib diisi untuk tipe akses password".into(),
                    data: None,
                    meta: None,
                });
            }
        };
        match hash(&pwd, DEFAULT_COST) {
            Ok(h) => Some(h),
            Err(_) => {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: "Gagal mengenkripsi password".into(),
                    data: None,
                    meta: None,
                });
            }
        }
    } else {
        None
    };

    let result = sqlx::query(
        "INSERT INTO forum_threads (user_id, title, content, tag_id, access_type, password_hash)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(&title)
    .bind(&content)
    .bind(form.tag_id)
    .bind(&access_type)
    .bind(&password_hash)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(res) => {
            let new_id = res.last_insert_id() as Int;
            Response::Ok().json(ApiResponse {
                success: true,
                message: "Thread berhasil dibuat".into(),
                data: Some(json!({ "id": new_id })),
                meta: None,
            })
        }
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "Gagal membuat thread".into(),
            data: None,
            meta: None,
        }),
    }
}

// ============================================================
// API — Edit Thread (login required, hanya pemilik)
// ============================================================

pub async fn edit_thread(
    pool: Pool,
    session: Session,
    path: Path<PathId>,
    form: Json<EditThreadForm>,
) -> Response {
    let user_id = auth!(session);
    let thread_id = path.id;

    // Gunakan ThreadAuthRow yang punya field password_hash
    let owner = match sqlx::query_as::<_, ThreadAuthRow>(
        "SELECT id, user_id AS author_id, access_type, password_hash
         FROM forum_threads WHERE id = ?",
    )
    .bind(thread_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Thread tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
    };

    if owner.author_id != user_id {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "Anda tidak memiliki akses".into(),
            data: None,
            meta: None,
        });
    }

    let title = form.title.as_deref().map(|s| s.trim()).unwrap_or("");
    let content = form.content.as_deref().map(|s| s.trim()).unwrap_or("");

    if title.is_empty() && content.is_empty() && form.tag_id.is_none() && form.access_type.is_none()
    {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Tidak ada data yang diubah".into(),
            data: None,
            meta: None,
        });
    }

    // Tentukan access_type dan password_hash baru
    let new_access = match &form.access_type {
        Some(a) if a == "public" || a == "user" || a == "password" => a.clone(),
        Some(_) => {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: "Tipe akses tidak valid".into(),
                data: None,
                meta: None,
            });
        }
        None => owner.access_type,
    };

    let new_hash = if new_access == "password" {
        match &form.password {
            Some(p) if !p.is_empty() => match hash(p, DEFAULT_COST) {
                Ok(h) => Some(h),
                Err(_) => {
                    return Response::InternalServerError().json(ApiResponse {
                        success: false,
                        message: "Gagal mengenkripsi password".into(),
                        data: None,
                        meta: None,
                    });
                }
            },
            _ => owner.password_hash,
        }
    } else {
        None
    };

    let result = sqlx::query(
        "UPDATE forum_threads
         SET title = COALESCE(NULLIF(?, ''), title),
             content = COALESCE(NULLIF(?, ''), content),
             tag_id = COALESCE(?, tag_id),
             access_type = ?,
             password_hash = ?
         WHERE id = ?",
    )
    .bind(form.title.as_deref().map(|s| s.trim()))
    .bind(form.content.as_deref().map(|s| s.trim()))
    .bind(form.tag_id)
    .bind(&new_access)
    .bind(&new_hash)
    .bind(thread_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Thread berhasil diperbarui".into(),
            data: Some(json!({ "id": thread_id })),
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "Gagal memperbarui thread".into(),
            data: None,
            meta: None,
        }),
    }
}

// ============================================================
// API — Delete Thread (login required, hanya pemilik)
// ============================================================

pub async fn delete_thread(pool: Pool, session: Session, path: Path<PathId>) -> Response {
    let user_id = auth!(session);
    let thread_id = path.id;

    let owner = match sqlx::query_as::<_, ThreadAuthRow>(
        "SELECT id, user_id AS author_id, access_type, password_hash
         FROM forum_threads WHERE id = ?",
    )
    .bind(thread_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Thread tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
    };

    if owner.author_id != user_id {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "Anda tidak memiliki akses".into(),
            data: None,
            meta: None,
        });
    }

    match sqlx::query("DELETE FROM forum_threads WHERE id = ?")
        .bind(thread_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Thread berhasil dihapus".into(),
            data: Some(json!({ "id": thread_id })),
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "Gagal menghapus thread".into(),
            data: None,
            meta: None,
        }),
    }
}

// ============================================================
// API — Create Reply (login required)
// ============================================================

pub async fn create_reply(
    pool: Pool,
    session: Session,
    path: Path<PathId>,
    form: Json<ReplyForm>,
) -> Response {
    let user_id = auth!(session);
    let thread_id = path.id;
    let content = form.content.trim().to_string();

    if content.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Balasan tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    // Pastikan thread ada
    let exists = sqlx::query("SELECT id FROM forum_threads WHERE id = ?")
        .bind(thread_id)
        .fetch_optional(pool.get_ref())
        .await;

    match exists {
        Ok(None) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Thread tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "Gagal memverifikasi thread".into(),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    let result = sqlx::query(
        "INSERT INTO forum_replies (thread_id, user_id, content) VALUES (?, ?, ?)",
    )
    .bind(thread_id)
    .bind(user_id)
    .bind(&content)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(res) => {
            let new_id = res.last_insert_id() as Int;
            Response::Ok().json(ApiResponse {
                success: true,
                message: "Balasan berhasil dikirim".into(),
                data: Some(json!({ "id": new_id })),
                meta: None,
            })
        }
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "Gagal mengirim balasan".into(),
            data: None,
            meta: None,
        }),
    }
}

// ============================================================
// API — Delete Reply (login required, hanya pemilik)
// ============================================================

pub async fn delete_reply(pool: Pool, session: Session, path: Path<PathId>) -> Response {
    let user_id = auth!(session);
    let reply_id = path.id;

    let reply = match sqlx::query_as::<_, ReplyRow>(
        "SELECT id, user_id AS author_id, content, created_at FROM forum_replies WHERE id = ?",
    )
    .bind(reply_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Balasan tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
    };

    if reply.author_id != user_id {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "Anda tidak memiliki akses".into(),
            data: None,
            meta: None,
        });
    }

    match sqlx::query("DELETE FROM forum_replies WHERE id = ?")
        .bind(reply_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Balasan berhasil dihapus".into(),
            data: Some(json!({ "id": reply_id })),
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "Gagal menghapus balasan".into(),
            data: None,
            meta: None,
        }),
    }
}