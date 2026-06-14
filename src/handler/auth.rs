use crate::web::{Pool, Session, Response, ApiResponse};
use crate::web::types::{Int, String, Date};
use crate::web::data::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::json;

#[derive(Deserialize)]
pub struct Logindata {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: Int,
    pub username: String,
    pub password: String,
    pub created_at: Date,
}

pub async fn login(pool: Pool, data: Json<Logindata>, session: Session) -> Response {

    let username = data.username.trim();
    let password = &data.password;

    if username.is_empty() || password.is_empty() {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "blank_input".into(),
            data: None,
            meta: None,
        });
    }

    let conn = pool.get_ref();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? LIMIT 1")
    .bind(username)
    .fetch_optional(conn)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None)    => return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "user_not_found".into(),
            data: None,
            meta: None,
        }),
        Err(_) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "db_error".into(),
            data: None,
            meta: None,
        }),
    };
    
    if !verify(password, &user.password).unwrap_or(false) {
        return Response::Unauthorized().json(ApiResponse {
            success: false,
            message: "password_not_match".into(),
            data: None,
            meta: None,
        });
    }

    if session.insert("user_id", user.id).is_err() {
        return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "session_error".into(),
            data: None,
            meta: None,
        });
    }

    Response::Ok().json(ApiResponse {
        success: true,
        message: "login_success".into(),
        data: Some(json!({
            "id": user.id,
            "username": user.username,
        })),
        meta: None,
    })
}

pub async fn register(pool: Pool, data: Json<Logindata>, session: Session) -> Response {

    let username = data.username.trim();
    let password = &data.password;

    if username.is_empty() || password.is_empty() {
        return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "blank_input".into(),
            data: None,
            meta: None,
        });
    }

    let conn = pool.get_ref();

    let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? LIMIT 1")
    .bind(username)
    .fetch_optional(conn)
    .await;

    match existing {
        Ok(Some(_)) => return Response::Conflict().json(ApiResponse {
            success: false,
            message: "username_already_used".into(),
            data: None,
            meta: None,
        }),
        Ok(None) => {},
        Err(_)   => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "db_error".into(),
            data: None,
            meta: None,
        }),
    }

    let hashed = match hash(password, DEFAULT_COST) {
        Ok(h)  => h,
        Err(_) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "hash_error".into(),
            data: None,
            meta: None,
        }),
    };

    let inserted = sqlx::query(
        "INSERT INTO users (username, password) VALUES (?, ?)"
    )
    .bind(username)
    .bind(&hashed)
    .execute(conn)
    .await;

    let inserted = match inserted {
        Ok(r)  => r,
        Err(_) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "db_error".into(),
            data: None,
            meta: None,
        }),
    };

    let new_id = inserted.last_insert_id() as Int;

    if session.insert("user_id", new_id).is_err() {
        return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: "session_error".into(),
            data: None,
            meta: None,
        });
    }

    Response::Created().json(ApiResponse {
        success: true,
        message: "register_success".into(),
        data: Some(json!({
            "id": new_id,
            "username": username,
        })),
        meta: None,
    })
}

pub async fn logout(session: Session) -> Response {
    session.purge();
    Response::Ok().json(ApiResponse {
        success: true,
        message: "logged_out".into(),
        data: None,
        meta: None,
    })
}