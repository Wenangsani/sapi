use crate::web::{Pool, Session, Response, ApiResponse};
use crate::web::from::{ Path, Json };
use crate::web::data::{ Int, UInt, String, Date };

use serde_json::json;

// =========================================================
// STRUCTS
// =========================================================

#[derive(FromRow, Serialize)]
pub struct ThreadListItem {
    pub id: UInt,
    pub title: String,
    pub access_type: String,
    pub view_count: UInt,
    pub reply_count: UInt,
    pub created_at: Date,
    pub author_name: String,
    pub tags: Option<String>, // hasil GROUP_CONCAT, dipecah saat serialisasi response
}

#[derive(FromRow, Serialize)]
pub struct TagItem {
    pub id: UInt,
    pub name: String,
    pub slug: String,
    pub thread_count: Int,
}

#[derive(FromRow, Serialize)]
pub struct ThreadDetail {
    pub id: UInt,
    pub user_id: UInt,
    pub title: String,
    pub content: String,
    pub access_type: String,
    pub view_count: UInt,
    pub reply_count: UInt,
    pub created_at: Date,
    pub author_name: String,
}

#[derive(FromRow, Serialize)]
pub struct ReplyItem {
    pub id: UInt,
    pub user_id: UInt,
    pub content: String,
    pub created_at: Date,
    pub author_name: String,
}

#[derive(Deserialize)]
pub struct ListThreadsQuery {
    pub tag: Option<String>,
    pub search: Option<String>,
    pub page: Option<UInt>,
    pub limit: Option<UInt>,
}

#[derive(Deserialize)]
pub struct CreateThreadBody {
    pub title: String,
    pub content: String,
    pub access_type: String, // "public" | "user" | "password"
    pub access_password: Option<String>,
    pub tags: Option<Vec<String>>, // nama tag, akan dibuat otomatis jika belum ada
}

#[derive(Deserialize)]
pub struct UnlockThreadBody {
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateReplyBody {
    pub content: String,
}

#[derive(Deserialize)]
pub struct ListRepliesQuery {
    pub page: Option<UInt>,
    pub limit: Option<UInt>,
}

// =========================================================
// HALAMAN
// =========================================================

/// GET /forum - halaman list thread (two-column desktop, single column mobile)
pub async fn page_list() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_list.html"))
}

/// GET /forum/{id} - halaman detail thread
pub async fn page_detail() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_detail.html"))
}

// =========================================================
// TAGS
// =========================================================

/// GET /forum/api/tags - daftar tag beserta jumlah thread (untuk sidebar)
pub async fn list_tags(pool: Pool) -> Response {
    let result = sqlx::query_as::<_, TagItem>(
        r#"
        SELECT
            t.id,
            t.name,
            t.slug,
            CAST(COUNT(tt.thread_id) AS SIGNED) AS thread_count
        FROM forum_tags t
        LEFT JOIN forum_thread_tags tt ON tt.tag_id = t.id
        GROUP BY t.id, t.name, t.slug
        ORDER BY thread_count DESC, t.name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    let tags = match result {
        Ok(rows) => rows,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_mengambil_data_tag".into(),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "berhasil_mengambil_data_tag".into(),
        data: Some(json!({ "tags": tags })),
        meta: None,
    })
}

// =========================================================
// THREAD LIST (dengan search, filter tag, pagination)
// =========================================================

/// GET /forum/api/threads?tag=&search=&page=&limit=
pub async fn list_threads(pool: Pool, query: actix_web::web::Query<ListThreadsQuery>) -> Response {
    let page: i64 = query.page.unwrap_or(1).max(1) as i64;
    let limit: i64 = query.limit.unwrap_or(20).clamp(1, 100) as i64;
    let offset: i64 = (page - 1) * limit;

    let search_pattern = query
        .search
        .as_ref()
        .map(|s| std::format!("%{}%", s.trim()))
        .unwrap_or_else(|| "%".to_string());

    let tag_slug = query.tag.clone().unwrap_or_default();
    let has_tag_filter = !tag_slug.is_empty();

    let rows_result = if has_tag_filter {
        sqlx::query_as::<_, ThreadListItem>(
            r#"
            SELECT
                th.id,
                th.title,
                th.access_type,
                th.view_count,
                th.reply_count,
                th.created_at,
                u.fullname AS author_name,
                (
                    SELECT GROUP_CONCAT(t2.name SEPARATOR ',')
                    FROM forum_thread_tags tt2
                    JOIN forum_tags t2 ON t2.id = tt2.tag_id
                    WHERE tt2.thread_id = th.id
                ) AS tags
            FROM forum_threads th
            JOIN users u ON u.id = th.user_id
            JOIN forum_thread_tags tt ON tt.thread_id = th.id
            JOIN forum_tags t ON t.id = tt.tag_id AND t.slug = ?
            WHERE th.title LIKE ?
            ORDER BY th.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(tag_slug)
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, ThreadListItem>(
            r#"
            SELECT
                th.id,
                th.title,
                th.access_type,
                th.view_count,
                th.reply_count,
                th.created_at,
                u.fullname AS author_name,
                (
                    SELECT GROUP_CONCAT(t2.name SEPARATOR ',')
                    FROM forum_thread_tags tt2
                    JOIN forum_tags t2 ON t2.id = tt2.tag_id
                    WHERE tt2.thread_id = th.id
                ) AS tags
            FROM forum_threads th
            JOIN users u ON u.id = th.user_id
            WHERE th.title LIKE ?
            ORDER BY th.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
    };

    let threads = match rows_result {
        Ok(rows) => rows,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_mengambil_data_thread".into(),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "berhasil_mengambil_data_thread".into(),
        data: Some(json!({ "threads": threads })),
        meta: Some(json!({ "page": page, "limit": limit })),
    })
}

// =========================================================
// CREATE THREAD
// =========================================================

/// POST /forum/api/threads - butuh login, dicek manual via auth!()
pub async fn create_thread(
    pool: Pool,
    session: Session,
    body: Json<CreateThreadBody>,
) -> Response {
    let user_id = auth!(session);
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return Response::Unauthorized().json(ApiResponse {
                success: false,
                message: "harus_login_terlebih_dahulu".into(),
                data: None,
                meta: None,
            });
        }
    };

    let title = body.title.trim().to_string();
    let content = body.content.trim().to_string();

    if title.is_empty() || content.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "judul_dan_konten_wajib_diisi".into(),
            data: None,
            meta: None,
        });
    }

    let access_type = match body.access_type.as_str() {
        "public" | "user" | "password" => body.access_type.clone(),
        _ => {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: "tipe_akses_tidak_valid".into(),
                data: None,
                meta: None,
            });
        }
    };

    let access_password_hash: Option<String> = if access_type == "password" {
        match &body.access_password {
            Some(pw) if !pw.is_empty() => match bcrypt::hash(pw, bcrypt::DEFAULT_COST) {
                Ok(hash) => Some(hash),
                Err(_) => {
                    return Response::InternalServerError().json(ApiResponse {
                        success: false,
                        message: "gagal_memproses_password".into(),
                        data: None,
                        meta: None,
                    });
                }
            },
            _ => {
                return Response::BadRequest().json(ApiResponse {
                    success: false,
                    message: "password_wajib_diisi_untuk_akses_password".into(),
                    data: None,
                    meta: None,
                });
            }
        }
    } else {
        None
    };

    let insert_result = sqlx::query(
        r#"
        INSERT INTO forum_threads (user_id, title, content, access_type, access_password)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&title)
    .bind(&content)
    .bind(&access_type)
    .bind(&access_password_hash)
    .execute(pool.get_ref())
    .await;

    let thread_id = match insert_result {
        Ok(res) => res.last_insert_id() as UInt,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_membuat_thread".into(),
                data: None,
                meta: None,
            });
        }
    };

    if let Some(tag_names) = &body.tags {
        for raw_name in tag_names {
            let name = raw_name.trim();
            if name.is_empty() {
                continue;
            }
            let slug = name.to_lowercase().replace(' ', "-");

            let tag_id_result = sqlx::query_as::<_, (UInt,)>(
                "SELECT id FROM forum_tags WHERE slug = ?",
            )
            .bind(&slug)
            .fetch_optional(pool.get_ref())
            .await;

            let tag_id: Option<UInt> = match tag_id_result {
                Ok(Some((id,))) => Some(id),
                Ok(None) => {
                    let create_tag = sqlx::query(
                        "INSERT INTO forum_tags (name, slug) VALUES (?, ?)",
                    )
                    .bind(name)
                    .bind(&slug)
                    .execute(pool.get_ref())
                    .await;

                    match create_tag {
                        Ok(res) => Some(res.last_insert_id() as UInt),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            };

            if let Some(tid) = tag_id {
                let _ = sqlx::query(
                    "INSERT IGNORE INTO forum_thread_tags (thread_id, tag_id) VALUES (?, ?)",
                )
                .bind(thread_id)
                .bind(tid)
                .execute(pool.get_ref())
                .await;
            }
        }
    }

    Response::Ok().json(ApiResponse {
        success: true,
        message: "thread_berhasil_dibuat".into(),
        data: Some(json!({ "id": thread_id })),
        meta: None,
    })
}

// =========================================================
// THREAD DETAIL (cek access type)
// =========================================================

/// GET /forum/api/threads/{id}
pub async fn get_thread_detail(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let thread_id = path.into_inner();

    let thread_result = sqlx::query_as::<_, ThreadDetail>(
        r#"
        SELECT
            th.id,
            th.user_id,
            th.title,
            th.content,
            th.access_type,
            th.view_count,
            th.reply_count,
            th.created_at,
            u.fullname AS author_name
        FROM forum_threads th
        JOIN users u ON u.id = th.user_id
        WHERE th.id = ?
        "#,
    )
    .bind(thread_id)
    .fetch_optional(pool.get_ref())
    .await;

    let thread = match thread_result {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "thread_tidak_ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_mengambil_data_thread".into(),
                data: None,
                meta: None,
            });
        }
    };

    // Cek hak akses
    let current_user_id: Option<UInt> = session.get::<UInt>("user_id").unwrap_or(None);

    match thread.access_type.as_str() {
        "public" => {}
        "user" => {
            if current_user_id.is_none() {
                return Response::Unauthorized().json(ApiResponse {
                    success: false,
                    message: "thread_ini_hanya_untuk_user_login".into(),
                    data: None,
                    meta: None,
                });
            }
        }
        "password" => {
            let is_owner = current_user_id == Some(thread.user_id);

            if !is_owner {
                let uid = match current_user_id {
                    Some(id) => id,
                    None => {
                        return Response::Unauthorized().json(ApiResponse {
                            success: false,
                            message: "thread_ini_dilindungi_password".into(),
                            data: None,
                            meta: None,
                        });
                    }
                };

                let unlocked = sqlx::query_as::<_, (UInt,)>(
                    "SELECT thread_id FROM forum_thread_unlocks WHERE thread_id = ? AND user_id = ?",
                )
                .bind(thread_id)
                .bind(uid)
                .fetch_optional(pool.get_ref())
                .await;

                match unlocked {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        return Response::Forbidden().json(ApiResponse {
                            success: false,
                            message: "thread_ini_dilindungi_password".into(),
                            data: None,
                            meta: None,
                        });
                    }
                    Err(_) => {
                        return Response::InternalServerError().json(ApiResponse {
                            success: false,
                            message: "gagal_memeriksa_akses_thread".into(),
                            data: None,
                            meta: None,
                        });
                    }
                }
            }
        }
        _ => {}
    }

    let _ = sqlx::query("UPDATE forum_threads SET view_count = view_count + 1 WHERE id = ?")
        .bind(thread_id)
        .execute(pool.get_ref())
        .await;

    Response::Ok().json(ApiResponse {
        success: true,
        message: "berhasil_mengambil_data_thread".into(),
        data: Some(json!({ "thread": thread })),
        meta: None,
    })
}

// =========================================================
// UNLOCK THREAD (password)
// =========================================================

/// POST /forum/api/threads/{id}/unlock - butuh login
pub async fn unlock_thread(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<UnlockThreadBody>,
) -> Response {
    let user_id = auth!(session);
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return Response::Unauthorized().json(ApiResponse {
                success: false,
                message: "harus_login_terlebih_dahulu".into(),
                data: None,
                meta: None,
            });
        }
    };

    let thread_id = path.into_inner();

    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT access_type, access_password FROM forum_threads WHERE id = ?",
    )
    .bind(thread_id)
    .fetch_optional(pool.get_ref())
    .await;

    let (access_type, password_hash) = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "thread_tidak_ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_mengambil_data_thread".into(),
                data: None,
                meta: None,
            });
        }
    };

    if access_type != "password" {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "thread_ini_tidak_memerlukan_password".into(),
            data: None,
            meta: None,
        });
    }

    let hash = match password_hash {
        Some(h) => h,
        None => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "konfigurasi_password_thread_tidak_valid".into(),
                data: None,
                meta: None,
            });
        }
    };

    let is_valid = bcrypt::verify(&body.password, &hash).unwrap_or(false);

    if !is_valid {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "password_salah".into(),
            data: None,
            meta: None,
        });
    }

    let insert_unlock = sqlx::query(
        "INSERT IGNORE INTO forum_thread_unlocks (thread_id, user_id) VALUES (?, ?)",
    )
    .bind(thread_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    if insert_unlock.is_err() {
        return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_menyimpan_status_akses".into(),
            data: None,
            meta: None,
        });
    }

    Response::Ok().json(ApiResponse {
        success: true,
        message: "thread_berhasil_dibuka".into(),
        data: None,
        meta: None,
    })
}

// =========================================================
// REPLIES
// =========================================================

/// GET /forum/api/threads/{id}/replies?page=&limit=
pub async fn list_replies(
    pool: Pool,
    path: Path<UInt>,
    query: actix_web::web::Query<ListRepliesQuery>,
) -> Response {
    let thread_id = path.into_inner();
    let page: i64 = query.page.unwrap_or(1).max(1) as i64;
    let limit: i64 = query.limit.unwrap_or(20).clamp(1, 100) as i64;
    let offset: i64 = (page - 1) * limit;

    let result = sqlx::query_as::<_, ReplyItem>(
        r#"
        SELECT
            r.id,
            r.user_id,
            r.content,
            r.created_at,
            u.fullname AS author_name
        FROM forum_replies r
        JOIN users u ON u.id = r.user_id
        WHERE r.thread_id = ?
        ORDER BY r.created_at ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(thread_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    let replies = match result {
        Ok(rows) => rows,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_mengambil_data_balasan".into(),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "berhasil_mengambil_data_balasan".into(),
        data: Some(json!({ "replies": replies })),
        meta: Some(json!({ "page": page, "limit": limit })),
    })
}

/// POST /forum/api/threads/{id}/replies - butuh login
pub async fn create_reply(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<CreateReplyBody>,
) -> Response {
    let user_id = auth!(session);
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return Response::Unauthorized().json(ApiResponse {
                success: false,
                message: "harus_login_terlebih_dahulu".into(),
                data: None,
                meta: None,
            });
        }
    };

    let thread_id = path.into_inner();
    let content = body.content.trim().to_string();

    if content.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "konten_balasan_tidak_boleh_kosong".into(),
            data: None,
            meta: None,
        });
    }

    let thread_exists = sqlx::query_as::<_, (UInt,)>("SELECT id FROM forum_threads WHERE id = ?")
        .bind(thread_id)
        .fetch_optional(pool.get_ref())
        .await;

    match thread_exists {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "thread_tidak_ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_memeriksa_thread".into(),
                data: None,
                meta: None,
            });
        }
    }

    let insert_result = sqlx::query(
        "INSERT INTO forum_replies (thread_id, user_id, content) VALUES (?, ?, ?)",
    )
    .bind(thread_id)
    .bind(user_id)
    .bind(&content)
    .execute(pool.get_ref())
    .await;

    let reply_id = match insert_result {
        Ok(res) => res.last_insert_id() as UInt,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_membuat_balasan".into(),
                data: None,
                meta: None,
            });
        }
    };

    let _ = sqlx::query("UPDATE forum_threads SET reply_count = reply_count + 1 WHERE id = ?")
        .bind(thread_id)
        .execute(pool.get_ref())
        .await;

    Response::Ok().json(ApiResponse {
        success: true,
        message: "balasan_berhasil_dibuat".into(),
        data: Some(json!({ "id": reply_id })),
        meta: None,
    })
}