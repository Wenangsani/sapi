use crate::web::{Pool, Session, Response, ApiResponse};
use crate::socketsession::SocketSession;
use crate::ssesession::SseSession;
use actix_web::web::Data;
use serde_json::json;

pub async fn page_dashboard(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_dashboard.html"))
}

pub async fn api_stats(session: Session, pool: Pool, socketlist: Data<SocketSession>, sselist: Data<SseSession>) -> Response {
    let _ = auth!(session);

    // --- Database stats ---
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

    // --- Connection counts ---
    let ws_count = socketlist.client_count();
    let sse_count = sselist.client_count();

    // --- Service health checks ---
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(pool.get_ref())
        .await
        .is_ok();

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!({
            "user_count": user_count.0,
            "file_count": file_count.0,
            "total_storage_bytes": total_size.0.unwrap_or(0),
            "ws_connections": ws_count,
            "sse_connections": sse_count,
            "services": {
                "api":      { "status": "online", "label": "API Server"  },
                "database": { "status": if db_ok { "online" } else { "error" }, "label": "Database" },
                "storage":  { "status": "online", "label": "Storage"     },
                "auth":     { "status": "online", "label": "Auth Service"},
                "websocket":{ "status": "online", "label": "WebSocket"   },
                "sse":      { "status": "online", "label": "SSE"         },
            }
        })),
        meta: None,
    })
}