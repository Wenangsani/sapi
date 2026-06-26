use crate::web::{Pool, Session, Response, ApiResponse};
use actix_web::web::Query;
use serde_json::json;
use crate::module::superadmin::superadmin_mod::{ActivityLog, LogsQuery};

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
        data: Some(json!({
            "logs": logs,
            "total": total,
            "page": page,
            "total_pages": total_pages,
        })),
        meta: None,
    })
}
