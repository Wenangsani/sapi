use crate::web::{Pool, Session, Response, ApiResponse};
use crate::web::from::{Path, Json};
use actix_web::web::Query;
use serde_json::json;
use bcrypt;
use crate::module::superadmin::superadmin_mod::{
    UserRow, PaginationQuery, EditUserInput, ChangePasswordInput,
};

pub async fn page_users(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_users.html"))
}

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

pub async fn api_edit_user(
    session: Session,
    pool: Pool,
    path: Path<(u32,)>,
    body: Json<EditUserInput>,
) -> Response {
    let _ = auth!(session);

    let target_id = path.into_inner().0;

    let fullname = body.fullname.trim().to_string();

    if fullname.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Nama lengkap tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    match sqlx::query!(
        "UPDATE users SET fullname = ? WHERE id = ?",
        fullname,
        target_id
    )
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
            message: "Data pengguna berhasil diperbarui".into(),
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

pub async fn api_change_password(
    session: Session,
    pool: Pool,
    path: Path<(u32,)>,
    body: Json<ChangePasswordInput>,
) -> Response {
    let _ = auth!(session);

    let target_id = path.into_inner().0;

    let new_password = body.new_password.trim().to_string();
    let confirm_password = body.confirm_password.trim().to_string();

    if new_password.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Password baru tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    if new_password.len() < 8 {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Password minimal 8 karakter".into(),
            data: None,
            meta: None,
        });
    }

    if new_password != confirm_password {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Konfirmasi password tidak cocok".into(),
            data: None,
            meta: None,
        });
    }

    let hashed = match bcrypt::hash(&new_password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal hash password: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    match sqlx::query!(
        "UPDATE users SET password = ? WHERE id = ?",
        hashed,
        target_id
    )
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
            message: "Password berhasil diubah".into(),
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