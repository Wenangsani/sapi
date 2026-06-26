use crate::web::{Pool, Session, Response, ApiResponse};
use crate::web::from::{Path, Json};
use actix_web::web::Query;
use serde_json::{json, Value};
use sqlx::{Column, Row};
use std::collections::HashMap;
use crate::module::superadmin::superadmin_mod::{DbQueryInput, PaginationQuery};

pub async fn page_database(session: Session) -> Response {
    let _ = auth!(session);
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_database.html"))
}

pub async fn api_db_query(
    session: Session,
    pool: Pool,
    body: Json<DbQueryInput>,
) -> Response {
    let _ = auth!(session);

    let sql_lower = body.sql.trim().to_lowercase();

    if !sql_lower.starts_with("select")
        && !sql_lower.starts_with("show")
        && !sql_lower.starts_with("describe")
    {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Hanya perintah SELECT, SHOW, dan DESCRIBE yang diizinkan".into(),
            data: None,
            meta: None,
        });
    }

    let sql: &'static str = Box::leak(body.sql.clone().into_boxed_str());

    match sqlx::query(sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let result: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let mut map = serde_json::Map::new();
                    for (i, col) in row.columns().iter().enumerate() {
                        let val = try_get_value(row, i);
                        map.insert(col.name().to_string(), val);
                    }
                    serde_json::Value::Object(map)
                })
                .collect();

            Response::Ok().json(ApiResponse {
                success: true,
                message: format!("{} baris ditemukan", result.len()),
                data: Some(json!(result)),
                meta: None,
            })
        }
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Query error: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn api_list_tables(session: Session, pool: Pool) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let sql: &'static str = "SHOW TABLES";
    match sqlx::query(sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let tables: Vec<String> = rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!({ "tables": tables })),
                meta: None,
            })
        }
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal mengambil tabel: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn api_table_columns(
    session: Session,
    pool: Pool,
    path: Path<(String,)>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let table = path.into_inner().0;
    let sql: &'static str = Box::leak(format!("DESCRIBE `{}`", table).into_boxed_str());

    match sqlx::query(sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let columns: Vec<Value> = rows
                .iter()
                .map(|row| {
                    json!({
                        "name": row.get::<String, _>(0),
                        "type": row.get::<String, _>(1),
                        "null": row.get::<String, _>(2),
                        "key": row.get::<String, _>(3),
                        "default": row.get::<Option<String>, _>(4),
                        "extra": row.get::<String, _>(5)
                    })
                })
                .collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!({ "columns": columns })),
                meta: None,
            })
        }
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal deskripsi tabel: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn api_table_rows(
    session: Session,
    pool: Pool,
    path: Path<(String,)>,
    query: Query<PaginationQuery>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let table = path.into_inner().0;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let count_sql: &'static str =
        Box::leak(format!("SELECT COUNT(*) FROM `{}`", table).into_boxed_str());
    let total: i64 = match sqlx::query_scalar(count_sql)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(c) => c,
        Err(e) => {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: format!("Gagal menghitung baris: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let data_sql: &'static str = Box::leak(
        format!("SELECT * FROM `{}` LIMIT {} OFFSET {}", table, limit, offset).into_boxed_str(),
    );

    let rows: Vec<Value> = match sqlx::query(data_sql).fetch_all(pool.get_ref()).await {
        Ok(rows) => rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, col) in row.columns().iter().enumerate() {
                    map.insert(col.name().to_string(), try_get_value(row, i));
                }
                Value::Object(map)
            })
            .collect(),
        Err(e) => {
            return Response::BadRequest().json(ApiResponse {
                success: false,
                message: format!("Gagal mengambil data: {}", e),
                data: None,
                meta: None,
            });
        }
    };

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!({
            "rows": rows,
            "page": page,
            "total_pages": total_pages,
            "total": total,
        })),
        meta: None,
    })
}

pub async fn api_insert_row(
    session: Session,
    pool: Pool,
    path: Path<(String,)>,
    body: Json<HashMap<String, Value>>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let table = path.into_inner().0;
    let data = body.into_inner();

    if data.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Data kosong".into(),
            data: None,
            meta: None,
        });
    }

    let columns: Vec<&String> = data.keys().collect();
    let values: Vec<String> = columns
        .iter()
        .map(|k| format!("'{}'", data[*k].as_str().unwrap_or("")))
        .collect();

    let sql: &'static str = Box::leak(
        format!(
            "INSERT INTO `{}` ({}) VALUES ({})",
            table,
            columns
                .iter()
                .map(|c| format!("`{}`", c))
                .collect::<Vec<_>>()
                .join(", "),
            values.join(", ")
        )
        .into_boxed_str(),
    );

    match sqlx::query(sql).execute(pool.get_ref()).await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Baris berhasil ditambahkan".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal insert: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn api_update_row(
    session: Session,
    pool: Pool,
    path: Path<(String, String)>,
    body: Json<HashMap<String, Value>>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let (table, id) = path.into_inner();
    let data = body.into_inner();

    if data.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Data kosong".into(),
            data: None,
            meta: None,
        });
    }

    let set_clause = data
        .iter()
        .map(|(k, v)| format!("`{}` = '{}'", k, v.as_str().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(", ");

    let sql: &'static str = Box::leak(
        format!(
            "UPDATE `{}` SET {} WHERE id = '{}'",
            table, set_clause, id
        )
        .into_boxed_str(),
    );

    match sqlx::query(sql).execute(pool.get_ref()).await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Baris berhasil diperbarui".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal update: {}", e),
            data: None,
            meta: None,
        }),
    }
}

pub async fn api_delete_row(
    session: Session,
    pool: Pool,
    path: Path<(String, String)>,
) -> Response {
    if auth!(session).is_none() {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "Silakan login terlebih dahulu".into(),
            data: None,
            meta: None,
        });
    }

    let (table, id) = path.into_inner();

    let sql: &'static str = Box::leak(
        format!("DELETE FROM `{}` WHERE id = '{}'", table, id).into_boxed_str(),
    );

    match sqlx::query(sql).execute(pool.get_ref()).await {
        Ok(_) => Response::Ok().json(ApiResponse {
            success: true,
            message: "Baris berhasil dihapus".into(),
            data: None,
            meta: None,
        }),
        Err(e) => Response::BadRequest().json(ApiResponse {
            success: false,
            message: format!("Gagal delete: {}", e),
            data: None,
            meta: None,
        }),
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

pub(super) fn try_get_value(row: &sqlx::mysql::MySqlRow, index: usize) -> Value {
    if let Ok(v) = row.try_get::<Option<String>, _>(index) {
        return v.map(|s| json!(s)).unwrap_or(json!(null));
    }
    if let Ok(v) = row.try_get::<Option<i64>, _>(index) {
        return v.map(|n| json!(n)).unwrap_or(json!(null));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(index) {
        return v.map(|f| json!(f)).unwrap_or(json!(null));
    }
    json!(null)
}
