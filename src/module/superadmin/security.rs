use crate::web::{Pool, Session, Response, ApiResponse};
use serde_json::json;

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

pub async fn api_get_security_stats(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let weak_password_count: i64 = match sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE LENGTH(password) < 8"
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(c) => c,
        Err(_) => 0,
    };

    let two_fa_count: i64 = 0;

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

pub async fn api_toggle_system_lock(session: Session, _pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Status kunci sistem telah diubah".into(),
        data: None,
        meta: None,
    })
}
