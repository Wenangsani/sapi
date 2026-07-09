use crate::web::{Data, Pool, Session, Cookie, Request, Response, ApiResponse};
use crate::web::from::{Path, Json, Form, Multipart, Socket, Sse};
use crate::web::data::{Int, UInt, Uuid, String, Date};

use actix_web::web::Query;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ================= STRUCTS =================

#[derive(Serialize, sqlx::FromRow)]
pub struct ConversationRow {
    pub id: UInt,
    pub partner_id: UInt,
    pub partner_name: String,
    pub last_message: Option<String>,
    pub last_message_at: Option<Date>,
    pub unread_count: Int,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MessageRow {
    pub id: UInt,
    pub sender_id: UInt,
    pub sender_name: String,
    pub content: String,
    pub created_at: Date,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserSearchRow {
    pub id: UInt,
    pub username: String,
    pub fullname: String,
}

#[derive(Deserialize)]
pub struct CreateConversationInput {
    pub partner_id: UInt,
}

#[derive(Deserialize)]
pub struct SendMessageInput {
    pub content: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

// ================= PAGE =================

pub async fn page(session: Session) -> Response {
    let user_id = session.get::<UInt>("user_id").ok().flatten();

    if user_id.is_none() {
        return Response::Found()
            .append_header(("Location", "/auth/login"))
            .finish();
    }

    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_messages.html"))
}

// ================= LIST CONVERSATIONS =================

pub async fn list_conversations(pool: Pool, session: Session) -> Response {
    let user_id = auth!(session).unwrap();

    let result = sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT
            c.id AS id,
            cp2.user_id AS partner_id,
            u.fullname AS partner_name,
            (SELECT m.content FROM messages m
                WHERE m.conversation_id = c.id AND m.is_deleted = 0
                ORDER BY m.created_at DESC LIMIT 1) AS last_message,
            (SELECT m.created_at FROM messages m
                WHERE m.conversation_id = c.id AND m.is_deleted = 0
                ORDER BY m.created_at DESC LIMIT 1) AS last_message_at,
            (SELECT COUNT(*) FROM messages m
                WHERE m.conversation_id = c.id AND m.is_deleted = 0
                AND m.sender_id != ?
                AND m.created_at > COALESCE(cp.last_read_at, '1970-01-01')) AS unread_count
        FROM conversation_participants cp
        JOIN conversations c ON c.id = cp.conversation_id
        JOIN conversation_participants cp2 ON cp2.conversation_id = c.id AND cp2.user_id != cp.user_id
        JOIN users u ON u.id = cp2.user_id
        WHERE cp.user_id = ? AND c.is_group = 0
        ORDER BY last_message_at DESC
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => Response::Ok().json(ApiResponse {
            success: true,
            message: "berhasil_mengambil_percakapan".into(),
            data: Some(json!({ "conversations": rows })),
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_mengambil_percakapan".into(),
            data: None,
            meta: None,
        }),
    }
}

// ================= CREATE / GET CONVERSATION =================

pub async fn create_conversation(
    pool: Pool,
    session: Session,
    body: Json<CreateConversationInput>,
) -> Response {
    let user_id = auth!(session).unwrap();
    let partner_id = body.partner_id;

    if partner_id == user_id {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "tidak_bisa_chat_diri_sendiri".into(),
            data: None,
            meta: None,
        });
    }

    let existing = sqlx::query_as::<_, (UInt,)>(
        r#"
        SELECT cp1.conversation_id
        FROM conversation_participants cp1
        JOIN conversation_participants cp2 ON cp1.conversation_id = cp2.conversation_id
        JOIN conversations c ON c.id = cp1.conversation_id
        WHERE cp1.user_id = ? AND cp2.user_id = ? AND c.is_group = 0
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(partner_id)
    .fetch_optional(pool.get_ref())
    .await;

    let existing_id = match existing {
        Ok(row) => row,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_memeriksa_percakapan".into(),
                data: None,
                meta: None,
            });
        }
    };

    if let Some((conv_id,)) = existing_id {
        return Response::Ok().json(ApiResponse {
            success: true,
            message: "percakapan_sudah_ada".into(),
            data: Some(json!({ "conversation_id": conv_id })),
            meta: None,
        });
    }

    let insert_conv = sqlx::query("INSERT INTO conversations (is_group) VALUES (0)")
        .execute(pool.get_ref())
        .await;

    let conv_id = match insert_conv {
        Ok(res) => res.last_insert_id() as UInt,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_membuat_percakapan".into(),
                data: None,
                meta: None,
            });
        }
    };

    let insert_participants = sqlx::query(
        "INSERT INTO conversation_participants (conversation_id, user_id) VALUES (?, ?), (?, ?)",
    )
    .bind(conv_id)
    .bind(user_id)
    .bind(conv_id)
    .bind(partner_id)
    .execute(pool.get_ref())
    .await;

    match insert_participants {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "percakapan_dibuat".into(),
            data: Some(json!({ "conversation_id": conv_id })),
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_menambahkan_peserta".into(),
            data: None,
            meta: None,
        }),
    }
}

// ================= GET MESSAGES =================

async fn is_participant(pool: &sqlx::MySqlPool, conversation_id: UInt, user_id: UInt) -> bool {
    sqlx::query("SELECT id FROM conversation_participants WHERE conversation_id = ? AND user_id = ?")
        .bind(conversation_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some()
}

pub async fn get_messages(pool: Pool, session: Session, path: Path<UInt>) -> Response {
    let user_id = auth!(session).unwrap();
    let conversation_id = path.into_inner();

    if !is_participant(pool.get_ref(), conversation_id, user_id).await {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "bukan_anggota_percakapan".into(),
            data: None,
            meta: None,
        });
    }

    let result = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT m.id, m.sender_id, u.fullname AS sender_name, m.content, m.created_at
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE m.conversation_id = ? AND m.is_deleted = 0
        ORDER BY m.created_at ASC
        LIMIT 200
        "#,
    )
    .bind(conversation_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let messages: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|m| {
                    json!({
                        "id": m.id,
                        "sender_id": m.sender_id,
                        "sender_name": m.sender_name,
                        "content": m.content,
                        "created_at": m.created_at,
                        "is_mine": m.sender_id == user_id,
                    })
                })
                .collect();

            Response::Ok().json(ApiResponse {
                success: true,
                message: "berhasil_mengambil_pesan".into(),
                data: Some(json!({ "messages": messages })),
                meta: None,
            })
        }
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_mengambil_pesan".into(),
            data: None,
            meta: None,
        }),
    }
}

// ================= SEND MESSAGE =================

pub async fn send_message(
    pool: Pool,
    session: Session,
    path: Path<UInt>,
    body: Json<SendMessageInput>,
) -> Response {
    let user_id = auth!(session).unwrap();
    let conversation_id = path.into_inner();
    let content = body.content.trim().to_string();

    if content.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "pesan_tidak_boleh_kosong".into(),
            data: None,
            meta: None,
        });
    }

    if !is_participant(pool.get_ref(), conversation_id, user_id).await {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "bukan_anggota_percakapan".into(),
            data: None,
            meta: None,
        });
    }

    let insert = sqlx::query(
        "INSERT INTO messages (conversation_id, sender_id, content) VALUES (?, ?, ?)",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(&content)
    .execute(pool.get_ref())
    .await;

    let message_id = match insert {
        Ok(res) => res.last_insert_id() as UInt,
        Err(_) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: "gagal_mengirim_pesan".into(),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "pesan_terkirim".into(),
        data: Some(json!({
            "id": message_id,
            "sender_id": user_id,
            "content": content,
        })),
        meta: None,
    })
}

// ================= MARK READ =================

pub async fn mark_read(pool: Pool, session: Session, path: Path<UInt>) -> Response {
    let user_id = auth!(session).unwrap();
    let conversation_id = path.into_inner();

    let result = sqlx::query(
        "UPDATE conversation_participants SET last_read_at = NOW() WHERE conversation_id = ? AND user_id = ?",
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "ditandai_terbaca".into(),
            data: None,
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_menandai_terbaca".into(),
            data: None,
            meta: None,
        }),
    }
}

// ================= SEARCH USERS =================

pub async fn search_users(pool: Pool, session: Session, query: Query<SearchQuery>) -> Response {
    let user_id = auth!(session).unwrap();
    let keyword = format!("%{}%", query.q.trim());

    if query.q.trim().is_empty() {
        return Response::Ok().json(ApiResponse {
            success: true,
            message: "berhasil_mencari_pengguna".into(),
            data: Some(json!({ "users": Vec::<UserSearchRow>::new() })),
            meta: None,
        });
    }

    let result = sqlx::query_as::<_, UserSearchRow>(
        "SELECT id, username, fullname FROM users WHERE (username LIKE ? OR fullname LIKE ?) AND id != ? LIMIT 20",
    )
    .bind(&keyword)
    .bind(&keyword)
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => Response::Ok().json(ApiResponse {
            success: true,
            message: "berhasil_mencari_pengguna".into(),
            data: Some(json!({ "users": rows })),
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_mencari_pengguna".into(),
            data: None,
            meta: None,
        }),
    }
}

// ================= DELETE MESSAGE =================

pub async fn delete_message(pool: Pool, session: Session, path: Path<UInt>) -> Response {
    let user_id = auth!(session).unwrap();
    let message_id = path.into_inner();

    let result = sqlx::query("UPDATE messages SET is_deleted = 1 WHERE id = ? AND sender_id = ?")
        .bind(message_id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => Response::Ok().json(ApiResponse {
            success: true,
            message: "pesan_dihapus".into(),
            data: None,
            meta: None,
        }),
        Ok(_) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: "pesan_tidak_ditemukan_atau_bukan_milik_anda".into(),
            data: None,
            meta: None,
        }),
        Err(_) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "gagal_menghapus_pesan".into(),
            data: None,
            meta: None,
        }),
    }
}