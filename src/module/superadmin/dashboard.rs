use crate::web::{Pool, Session, Response, ApiResponse};
use serde_json::json;

pub async fn page_dashboard(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_dashboard.html"))
}

pub async fn api_stats(session: Session, pool: Pool) -> Response {
    let _ = auth!(session);

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
