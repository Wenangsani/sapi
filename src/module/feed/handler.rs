// src/module/feed/handler.rs

use crate::web::{Data, Pool, Session, Response, ApiResponse};
use crate::web::from::{Path, Json};
use crate::web::data::{Int, UInt, String as Str};
use actix_web::HttpRequest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ========================================================
// STRUCTS
// ========================================================

#[derive(Debug, Serialize)]
pub struct PostItem {
    pub id: UInt,
    pub user_id: UInt,
    pub username: Str,
    pub fullname: Str,
    pub content: Str,
    pub image_url: Option<Str>,
    pub visibility: Str,
    pub like_count: i64,
    pub comment_count: i64,
    pub is_liked: bool,
    pub is_saved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommentItem {
    pub id: UInt,
    pub post_id: UInt,
    pub user_id: UInt,
    pub username: Str,
    pub fullname: Str,
    pub parent_id: Option<UInt>,
    pub content: Str,
    pub like_count: i64,
    pub is_liked: bool,
    pub created_at: DateTime<Utc>,
    pub replies: Vec<CommentItem>,
}

#[derive(Debug, Serialize)]
pub struct NotificationItem {
    pub id: UInt,
    pub actor_username: Str,
    pub actor_fullname: Str,
    pub notif_type: Str,
    pub post_id: Option<UInt>,
    pub comment_id: Option<UInt>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostInput {
    pub content: Str,
    pub image_url: Option<Str>,
    pub visibility: Option<Str>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostInput {
    pub content: Option<Str>,
    pub image_url: Option<Str>,
    pub visibility: Option<Str>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentInput {
    pub content: Str,
    pub parent_id: Option<UInt>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentInput {
    pub content: Str,
}

#[derive(Debug, Deserialize)]
pub struct ReportPostInput {
    pub reason: Str,
    pub note: Option<Str>,
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub page: Option<UInt>,
    pub limit: Option<UInt>,
}

// ========================================================
// HALAMAN
// ========================================================

pub async fn page_feed(session: Session) -> Response {
    let _user_id = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_feed.html"))
}

pub async fn page_post_detail(session: Session) -> Response {
    let _user_id = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_post.html"))
}

// ========================================================
// POSTS
// ========================================================

pub async fn get_posts(
    pool: Pool,
    session: Session,
    req: HttpRequest,
) -> Response {
    let user_id = auth!(session);

    // Ambil page & limit dari query string
    let query_str = req.query_string();
    let page: u64 = extract_query_u64(query_str, "page").unwrap_or(1).max(1);
    let limit: u64 = extract_query_u64(query_str, "limit").unwrap_or(10).min(50);
    let offset: u64 = (page - 1) * limit;

    // Hitung total
    let total_row = sqlx::query!(
        r#"SELECT COUNT(*) AS total FROM posts WHERE is_deleted = 0 AND visibility = 'public'"#
    )
    .fetch_one(pool.get_ref())
    .await;

    let total = match total_row {
        Ok(row) => row.total,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal menghitung post: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    // Fetch posts dengan like count, comment count, apakah di-like user ini, apakah di-save
    let rows = sqlx::query!(
        r#"
        SELECT
            p.id            AS post_id,
            p.user_id,
            u.username,
            u.fullname,
            p.content,
            p.image_url,
            p.visibility,
            p.created_at,
            p.updated_at,
            CAST(COALESCE(lc.like_count, 0) AS SIGNED)    AS like_count,
            CAST(COALESCE(cc.comment_count, 0) AS SIGNED) AS comment_count,
            CASE WHEN ul.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_liked,
            CASE WHEN us.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_saved
        FROM posts p
        JOIN users u ON u.id = p.user_id
        LEFT JOIN (
            SELECT post_id, COUNT(*) AS like_count FROM post_likes GROUP BY post_id
        ) lc ON lc.post_id = p.id
        LEFT JOIN (
            SELECT post_id, COUNT(*) AS comment_count FROM post_comments WHERE is_deleted = 0 GROUP BY post_id
        ) cc ON cc.post_id = p.id
        LEFT JOIN post_likes ul ON ul.post_id = p.id AND ul.user_id = ?
        LEFT JOIN post_saves us ON us.post_id = p.id AND us.user_id = ?
        WHERE p.is_deleted = 0 AND p.visibility = 'public'
        ORDER BY p.created_at DESC
        LIMIT ? OFFSET ?
        "#,
        user_id,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let posts: Vec<PostItem> = rows.into_iter().map(|r| PostItem {
                id: r.post_id,
                user_id: r.user_id,
                username: r.username,
                fullname: r.fullname,
                content: r.content,
                image_url: r.image_url,
                visibility: r.visibility,
                like_count: r.like_count,
                comment_count: r.comment_count,
                is_liked: r.is_liked == 1,
                is_saved: r.is_saved == 1,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
            }).collect();

            Response::Ok().json(ApiResponse {
                success: true,
                message: "Berhasil".into(),
                data: Some(json!({ "posts": posts })),
                meta: Some(json!({
                    "page": page,
                    "limit": limit,
                    "total": total,
                    "total_pages": (total as f64 / limit as f64).ceil() as u64
                })),
            })
        }
        Err(e) => {
            Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal mengambil posts: {}", e),
                data: None,
                meta: None,
            })
        }
    }
}

pub async fn get_post_detail(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    let row = sqlx::query!(
        r#"
        SELECT
            p.id AS post_id,
            p.user_id,
            u.username,
            u.fullname,
            p.content,
            p.image_url,
            p.visibility,
            p.created_at,
            p.updated_at,
            CAST(COALESCE(lc.like_count, 0) AS SIGNED) AS like_count,
            CAST(COALESCE(cc.comment_count, 0) AS SIGNED) AS comment_count,
            CASE WHEN ul.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_liked,
            CASE WHEN us.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_saved
        FROM posts p
        JOIN users u ON u.id = p.user_id
        LEFT JOIN (SELECT post_id, COUNT(*) AS like_count FROM post_likes GROUP BY post_id) lc ON lc.post_id = p.id
        LEFT JOIN (SELECT post_id, COUNT(*) AS comment_count FROM post_comments WHERE is_deleted = 0 GROUP BY post_id) cc ON cc.post_id = p.id
        LEFT JOIN post_likes ul ON ul.post_id = p.id AND ul.user_id = ?
        LEFT JOIN post_saves us ON us.post_id = p.id AND us.user_id = ?
        WHERE p.id = ? AND p.is_deleted = 0
        "#,
        user_id, user_id, post_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match row {
        Ok(Some(r)) => {
            let post = PostItem {
                id: r.post_id,
                user_id: r.user_id,
                username: r.username,
                fullname: r.fullname,
                content: r.content,
                image_url: r.image_url,
                visibility: r.visibility,
                like_count: r.like_count,
                comment_count: r.comment_count,
                is_liked: r.is_liked == 1,
                is_saved: r.is_saved == 1,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
            };
            Response::Ok().json(ApiResponse {
                success: true,
                message: "Berhasil".into(),
                data: Some(json!({ "post": post })),
                meta: None,
            })
        }
        Ok(None) => Response::NotFound().json(ApiResponse {
            success: false,
            message: "Post tidak ditemukan".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mengambil post: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn create_post(
    pool: Pool,
    session: Session,
    body: Json<CreatePostInput>,
) -> Response {
    let user_id = auth!(session);

    if body.content.trim().is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Konten post tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    let visibility = body.visibility.as_deref().unwrap_or("public");
    if !matches!(visibility, "public" | "friends" | "private") {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Nilai visibility tidak valid".into(),
            data: None,
            meta: None,
        });
    }

    let result = sqlx::query!(
        r#"INSERT INTO posts (user_id, content, image_url, visibility) VALUES (?, ?, ?, ?)"#,
        user_id,
        body.content.trim(),
        body.image_url.as_deref(),
        visibility
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Post berhasil dibuat".into(),
            data: Some(json!({ "post_id": r.last_insert_id() })),
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal membuat post: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn update_post(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<UpdatePostInput>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    // Cek post ada
    let exists = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM posts WHERE id = ? AND is_deleted = 0"#,
        post_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match exists {
        Ok(row) if row.cnt == 0 => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Post tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi post: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    // Cek kepemilikan
    let is_owner = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM posts WHERE id = ? AND user_id = ? AND is_deleted = 0"#,
        post_id, user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match is_owner {
        Ok(row) if row.cnt == 0 => {
            return Response::Forbidden().json(ApiResponse {
                success: false,
                message: "Kamu tidak berhak mengedit post ini".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi kepemilikan: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    if let Some(ref visibility) = body.visibility {
        if !matches!(visibility.as_str(), "public" | "friends" | "private") {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: "Nilai visibility tidak valid".into(),
                data: None,
                meta: None,
            });
        }
    }

    let result = sqlx::query!(
        r#"
        UPDATE posts
        SET
            content    = COALESCE(?, content),
            image_url  = COALESCE(?, image_url),
            visibility = COALESCE(?, visibility)
        WHERE id = ? AND is_deleted = 0
        "#,
        body.content.as_deref(),
        body.image_url.as_deref(),
        body.visibility.as_deref(),
        post_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Post berhasil diperbarui".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal memperbarui post: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn delete_post(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    // Cek post ada
    let exists = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM posts WHERE id = ? AND is_deleted = 0"#,
        post_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match exists {
        Ok(row) if row.cnt == 0 => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Post tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi post: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    // Cek kepemilikan
    let is_owner = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM posts WHERE id = ? AND user_id = ? AND is_deleted = 0"#,
        post_id, user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match is_owner {
        Ok(row) if row.cnt == 0 => {
            return Response::Forbidden().json(ApiResponse {
                success: false,
                message: "Kamu tidak berhak menghapus post ini".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi kepemilikan: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    let result = sqlx::query!(
        r#"UPDATE posts SET is_deleted = 1 WHERE id = ?"#,
        post_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Post berhasil dihapus".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal menghapus post: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ========================================================
// LIKES
// ========================================================

pub async fn toggle_like_post(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    // Cek apakah sudah di-like
    let existing = sqlx::query!(
        r#"SELECT id FROM post_likes WHERE post_id = ? AND user_id = ?"#,
        post_id, user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(_)) => {
            // Unlike
            match sqlx::query!(
                r#"DELETE FROM post_likes WHERE post_id = ? AND user_id = ?"#,
                post_id, user_id
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => Response::Ok().json(ApiResponse {
                    success: true,
                    message: "Like dibatalkan".into(),
                    data: Some(json!({ "liked": false })),
                    meta: None,
                }),
                Err(e) => Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal unlike: {}", e),
                    data: None,
                    meta: None,
                }),
            }
        }
        Ok(None) => {
            // Like & buat notifikasi ke pemilik post
            let tx = pool.get_ref().begin().await;
            match tx {
                Err(e) => Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal memulai transaksi: {}", e),
                    data: None,
                    meta: None,
                }),
                Ok(mut tx) => {
                    let insert = sqlx::query!(
                        r#"INSERT IGNORE INTO post_likes (post_id, user_id) VALUES (?, ?)"#,
                        post_id, user_id
                    )
                    .execute(&mut *tx)
                    .await;

                    if let Err(e) = insert {
                        let _ = tx.rollback().await;
                        return Response::InternalServerError().json(ApiResponse {
                            success: false,
                            message: format!("Gagal like: {}", e),
                            data: None,
                            meta: None,
                        });
                    }

                    // Cek apakah liker bukan owner post, lalu kirim notifikasi
                    let not_owner = sqlx::query!(
                        r#"SELECT COUNT(*) AS cnt FROM posts WHERE id = ? AND user_id != ? AND is_deleted = 0"#,
                        post_id, user_id
                    )
                    .fetch_one(&mut *tx)
                    .await;

                    if let Ok(row) = not_owner {
                        if row.cnt > 0 {
                            let owner_row = sqlx::query!(
                                r#"SELECT user_id FROM posts WHERE id = ? AND is_deleted = 0"#,
                                post_id
                            )
                            .fetch_optional(&mut *tx)
                            .await;

                            if let Ok(Some(_owner)) = owner_row {
                                let _ = sqlx::query!(
                                    r#"INSERT INTO notifications (user_id, actor_id, type, post_id)
                                       SELECT user_id, ?, 'like_post', id FROM posts WHERE id = ? AND is_deleted = 0"#,
                                    user_id, post_id
                                )
                                .execute(&mut *tx)
                                .await;
                            }
                        }
                    }

                    match tx.commit().await {
                        Ok(_) => Response::Ok().json(ApiResponse {
                            success: true,
                            message: "Post disukai".into(),
                            data: Some(json!({ "liked": true })),
                            meta: None,
                        }),
                        Err(e) => Response::InternalServerError().json(ApiResponse {
                            success: false,
                            message: format!("Gagal commit transaksi: {}", e),
                            data: None,
                            meta: None,
                        }),
                    }
                }
            }
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal cek like: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ========================================================
// SAVE / BOOKMARK
// ========================================================

pub async fn toggle_save_post(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    let existing = sqlx::query!(
        r#"SELECT id FROM post_saves WHERE post_id = ? AND user_id = ?"#,
        post_id, user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(_)) => {
            match sqlx::query!(
                r#"DELETE FROM post_saves WHERE post_id = ? AND user_id = ?"#,
                post_id, user_id
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => Response::Ok().json(ApiResponse {
                    success: true,
                    message: "Post dihapus dari simpanan".into(),
                    data: Some(json!({ "saved": false })),
                    meta: None,
                }),
                Err(e) => Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal hapus simpanan: {}", e),
                    data: None,
                    meta: None,
                }),
            }
        }
        Ok(None) => {
            match sqlx::query!(
                r#"INSERT IGNORE INTO post_saves (post_id, user_id) VALUES (?, ?)"#,
                post_id, user_id
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => Response::Ok().json(ApiResponse {
                    success: true,
                    message: "Post disimpan".into(),
                    data: Some(json!({ "saved": true })),
                    meta: None,
                }),
                Err(e) => Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal menyimpan post: {}", e),
                    data: None,
                    meta: None,
                }),
            }
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal cek simpanan: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn get_saved_posts(
    pool: Pool,
    session: Session,
    req: HttpRequest,
) -> Response {
    let user_id = auth!(session);
    let query_str = req.query_string();
    let page: u64 = extract_query_u64(query_str, "page").unwrap_or(1).max(1);
    let limit: u64 = extract_query_u64(query_str, "limit").unwrap_or(10).min(50);
    let offset: u64 = (page - 1) * limit;

    let rows = sqlx::query!(
        r#"
        SELECT
            p.id AS post_id,
            p.user_id,
            u.username,
            u.fullname,
            p.content,
            p.image_url,
            p.visibility,
            p.created_at,
            p.updated_at,
            CAST(COALESCE(lc.like_count, 0) AS SIGNED) AS like_count,
            CAST(COALESCE(cc.comment_count, 0) AS SIGNED) AS comment_count,
            CASE WHEN ul.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_liked,
            1 AS is_saved
        FROM post_saves ps
        JOIN posts p ON p.id = ps.post_id AND p.is_deleted = 0
        JOIN users u ON u.id = p.user_id
        LEFT JOIN (SELECT post_id, COUNT(*) AS like_count FROM post_likes GROUP BY post_id) lc ON lc.post_id = p.id
        LEFT JOIN (SELECT post_id, COUNT(*) AS comment_count FROM post_comments WHERE is_deleted = 0 GROUP BY post_id) cc ON cc.post_id = p.id
        LEFT JOIN post_likes ul ON ul.post_id = p.id AND ul.user_id = ?
        WHERE ps.user_id = ?
        ORDER BY ps.created_at DESC
        LIMIT ? OFFSET ?
        "#,
        user_id, user_id, limit, offset
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let posts: Vec<PostItem> = rows.into_iter().map(|r| PostItem {
                id: r.post_id,
                user_id: r.user_id,
                username: r.username,
                fullname: r.fullname,
                content: r.content,
                image_url: r.image_url,
                visibility: r.visibility,
                like_count: r.like_count,
                comment_count: r.comment_count,
                is_liked: r.is_liked == 1,
                is_saved: true,
                created_at: r.created_at.and_utc(),
                updated_at: r.updated_at.and_utc(),
            }).collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "Berhasil".into(),
                data: Some(json!({ "posts": posts })),
                meta: Some(json!({ "page": page, "limit": limit })),
            })
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mengambil post tersimpan: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ========================================================
// REPORT
// ========================================================

pub async fn report_post(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<ReportPostInput>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    let valid_reasons = ["spam", "harassment", "false_info", "violence", "other"];
    if !valid_reasons.contains(&body.reason.as_str()) {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Alasan tidak valid".into(),
            data: None,
            meta: None,
        });
    }

    let result = sqlx::query!(
        r#"INSERT IGNORE INTO post_reports (post_id, user_id, reason, note) VALUES (?, ?, ?, ?)"#,
        post_id, user_id, body.reason, body.note.as_deref()
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Laporan berhasil dikirim".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal melaporkan post: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ========================================================
// COMMENTS
// ========================================================

pub async fn get_comments(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    // Ambil semua komentar (parent + reply) sekaligus
    let rows = sqlx::query!(
        r#"
        SELECT
            c.id AS comment_id,
            c.post_id,
            c.user_id,
            u.username,
            u.fullname,
            c.parent_id,
            c.content,
            c.created_at,
            CAST(COALESCE(lc.like_count, 0) AS SIGNED) AS like_count,
            CASE WHEN cl.user_id IS NOT NULL THEN 1 ELSE 0 END AS is_liked
        FROM post_comments c
        JOIN users u ON u.id = c.user_id
        LEFT JOIN (SELECT comment_id, COUNT(*) AS like_count FROM comment_likes GROUP BY comment_id) lc ON lc.comment_id = c.id
        LEFT JOIN comment_likes cl ON cl.comment_id = c.id AND cl.user_id = ?
        WHERE c.post_id = ? AND c.is_deleted = 0
        ORDER BY c.created_at ASC
        "#,
        user_id, post_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            // Build tree: parent komentar & replies
            let mut parents: Vec<CommentItem> = Vec::new();
            let mut replies_map: std::collections::HashMap<UInt, Vec<CommentItem>> = std::collections::HashMap::new();

            for r in rows {
                let item = CommentItem {
                    id: r.comment_id,
                    post_id: r.post_id,
                    user_id: r.user_id,
                    username: r.username,
                    fullname: r.fullname,
                    parent_id: r.parent_id,
                    content: r.content,
                    like_count: r.like_count,
                    is_liked: r.is_liked == 1,
                    created_at: r.created_at.and_utc(),
                    replies: Vec::new(),
                };
                if let Some(pid) = item.parent_id {
                    replies_map.entry(pid).or_default().push(item);
                } else {
                    parents.push(item);
                }
            }

            // Attach replies ke parent
            let result: Vec<CommentItem> = parents.into_iter().map(|mut p| {
                if let Some(replies) = replies_map.remove(&p.id) {
                    p.replies = replies;
                }
                p
            }).collect();

            Response::Ok().json(ApiResponse {
                success: true,
                message: "Berhasil".into(),
                data: Some(json!({ "comments": result })),
                meta: None,
            })
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mengambil komentar: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn create_comment(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<CreateCommentInput>,
) -> Response {
    let user_id = auth!(session);
    let post_id = path.into_inner();

    if body.content.trim().is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Komentar tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    // Validasi parent_id jika ada
    if let Some(parent_id) = body.parent_id {
        let parent = sqlx::query!(
            r#"SELECT id FROM post_comments WHERE id = ? AND post_id = ? AND is_deleted = 0"#,
            parent_id, post_id
        )
        .fetch_optional(pool.get_ref())
        .await;

        match parent {
            Ok(None) => {
                return Response::BadRequest().json(ApiResponse {
                    success: false,
                    message: "Komentar induk tidak ditemukan".into(),
                    data: None,
                    meta: None,
                });
            }
            Err(e) => {
                return Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal validasi parent: {}", e),
                    data: None,
                    meta: None,
                });
            }
            _ => {}
        }
    }

    let mut tx = match pool.get_ref().begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal memulai transaksi: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let insert = sqlx::query!(
        r#"INSERT INTO post_comments (post_id, user_id, parent_id, content) VALUES (?, ?, ?, ?)"#,
        post_id, user_id, body.parent_id, body.content.trim()
    )
    .execute(&mut *tx)
    .await;

    let comment_id = match insert {
        Ok(r) => r.last_insert_id() as UInt,
        Err(e) => {
            let _ = tx.rollback().await;
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal membuat komentar: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    // Notifikasi ke owner post (jika bukan diri sendiri)
    let _ = sqlx::query!(
        r#"INSERT INTO notifications (user_id, actor_id, type, post_id, comment_id)
           SELECT p.user_id, ?, ?, p.id, ?
           FROM posts p
           WHERE p.id = ? AND p.is_deleted = 0 AND p.user_id != ?"#,
        user_id,
        if body.parent_id.is_some() { "reply" } else { "comment" },
        comment_id,
        post_id,
        user_id
    )
    .execute(&mut *tx)
    .await;

    match tx.commit().await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Komentar berhasil dibuat".into(),
            data: Some(json!({ "comment_id": comment_id })),
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal commit: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn update_comment(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<UpdateCommentInput>,
) -> Response {
    let user_id = auth!(session);
    let comment_id = path.into_inner();

    if body.content.trim().is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Komentar tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    // FIX: Hindari perbandingan tipe user_id yang ambigu dengan COUNT query
    let exists = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM post_comments WHERE id = ? AND is_deleted = 0"#,
        comment_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match exists {
        Ok(row) if row.cnt == 0 => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Komentar tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi komentar: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    let is_owner = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM post_comments WHERE id = ? AND user_id = ? AND is_deleted = 0"#,
        comment_id, user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match is_owner {
        Ok(row) if row.cnt == 0 => {
            return Response::Forbidden().json(ApiResponse {
                success: false,
                message: "Kamu tidak berhak mengedit komentar ini".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi kepemilikan: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    let result = sqlx::query!(
        r#"UPDATE post_comments SET content = ? WHERE id = ? AND is_deleted = 0"#,
        body.content.trim(), comment_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Komentar diperbarui".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal memperbarui komentar: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn delete_comment(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let comment_id = path.into_inner();

    // FIX: Hindari perbandingan tipe user_id yang ambigu dengan COUNT query
    let exists = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM post_comments WHERE id = ? AND is_deleted = 0"#,
        comment_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match exists {
        Ok(row) if row.cnt == 0 => {
            return Response::NotFound().json(ApiResponse {
                success: false,
                message: "Komentar tidak ditemukan".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi komentar: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    let is_owner = sqlx::query!(
        r#"SELECT COUNT(*) AS cnt FROM post_comments WHERE id = ? AND user_id = ? AND is_deleted = 0"#,
        comment_id, user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match is_owner {
        Ok(row) if row.cnt == 0 => {
            return Response::Forbidden().json(ApiResponse {
                success: false,
                message: "Kamu tidak berhak menghapus komentar ini".into(),
                data: None,
                meta: None,
            });
        }
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal verifikasi kepemilikan: {}", e),
                data: None,
                meta: None,
            });
        }
        _ => {}
    }

    let result = sqlx::query!(
        r#"UPDATE post_comments SET is_deleted = 1 WHERE id = ?"#,
        comment_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Komentar dihapus".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal menghapus komentar: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn toggle_like_comment(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let comment_id = path.into_inner();

    let existing = sqlx::query!(
        r#"SELECT id FROM comment_likes WHERE comment_id = ? AND user_id = ?"#,
        comment_id, user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing {
        Ok(Some(_)) => {
            match sqlx::query!(
                r#"DELETE FROM comment_likes WHERE comment_id = ? AND user_id = ?"#,
                comment_id, user_id
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => Response::Ok().json(ApiResponse {
                    success: true,
                    message: "Like dibatalkan".into(),
                    data: Some(json!({ "liked": false })),
                    meta: None,
                }),
                Err(e) => Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal unlike komentar: {}", e),
                    data: None,
                    meta: None,
                }),
            }
        }
        Ok(None) => {
            match sqlx::query!(
                r#"INSERT IGNORE INTO comment_likes (comment_id, user_id) VALUES (?, ?)"#,
                comment_id, user_id
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(_) => Response::Ok().json(ApiResponse {
                    success: true,
                    message: "Komentar disukai".into(),
                    data: Some(json!({ "liked": true })),
                    meta: None,
                }),
                Err(e) => Response::InternalServerError().json(ApiResponse {
                    success: false,
                    message: format!("Gagal like komentar: {}", e),
                    data: None,
                    meta: None,
                }),
            }
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal cek like komentar: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ========================================================
// NOTIFIKASI
// ========================================================

pub async fn get_notifications(
    pool: Pool,
    session: Session,
) -> Response {
    let user_id = auth!(session);

    let rows = sqlx::query!(
        r#"
        SELECT
            n.id AS notif_id,
            u.username AS actor_username,
            u.fullname AS actor_fullname,
            n.type AS notif_type,
            n.post_id,
            n.comment_id,
            n.is_read,
            n.created_at
        FROM notifications n
        JOIN users u ON u.id = n.actor_id
        WHERE n.user_id = ?
        ORDER BY n.created_at DESC
        LIMIT 50
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let notifs: Vec<NotificationItem> = rows.into_iter().map(|r| NotificationItem {
                id: r.notif_id,
                actor_username: r.actor_username,
                actor_fullname: r.actor_fullname,
                notif_type: r.notif_type,
                post_id: r.post_id,
                comment_id: r.comment_id,
                is_read: r.is_read == 1,
                created_at: r.created_at.and_utc(),
            }).collect();

            let unread = notifs.iter().filter(|n| !n.is_read).count();

            Response::Ok().json(ApiResponse {
                success: true,
                message: "Berhasil".into(),
                data: Some(json!({ "notifications": notifs, "unread": unread })),
                meta: None,
            })
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mengambil notifikasi: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn mark_read(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
) -> Response {
    let user_id = auth!(session);
    let notif_id = path.into_inner();

    let result = sqlx::query!(
        r#"UPDATE notifications SET is_read = 1 WHERE id = ? AND user_id = ?"#,
        notif_id, user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Notifikasi ditandai terbaca".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal update notifikasi: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn mark_all_read(
    pool: Pool,
    session: Session,
) -> Response {
    let user_id = auth!(session);

    let result = sqlx::query!(
        r#"UPDATE notifications SET is_read = 1 WHERE user_id = ? AND is_read = 0"#,
        user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) => Response::Ok().json(ApiResponse {
            success: true,
            message: format!("{} notifikasi ditandai terbaca", r.rows_affected()),
            data: None,
            meta: None,
        }),
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal update notifikasi: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ========================================================
// HELPER
// ========================================================

fn extract_query_u64(query_str: &str, key: &str) -> Option<u64> {
    for pair in query_str.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == key {
                return v.parse::<u64>().ok();
            }
        }
    }
    None
}