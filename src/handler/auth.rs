use crate::web::{Pool, Response, Warning};
use crate::web::types::{Int, String, Date};
use crate::web::data::Json;
use actix_session::Session;
use bcrypt::{hash, verify, DEFAULT_COST};

fn default_value() -> String {
    String::from("")
}

#[derive(Deserialize)]
pub struct Logindata {
    #[serde(default = "default_value")]
    pub email: String,
    #[serde(default = "default_value")]
    pub password: String,
}

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: Int,
    pub email: String,
    pub password: String,
    pub created_at: Date,
}

#[derive(Serialize)]
pub struct Output {
    pub id: Int,
    pub email: String,
}

pub async fn login(pool: Pool, data: Json<Logindata>, session: Session) -> Response {

    let email    = data.email.trim();
    let password = &data.password;

    // Cek input kosong
    if email.is_empty() || password.is_empty() {
        return Response::Forbidden().json(Warning { message: "blank_input" });
    }

    let conn = pool.get_ref();

    // Gunakan fetch_optional — lebih efisien untuk cek 1 record
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ? LIMIT 1"
    )
    .bind(email)
    .fetch_optional(conn)
    .await;

    // Tangani error koneksi DB
    let user = match user {
        Ok(Some(u)) => u,
        Ok(None)    => return Response::Unauthorized().json(Warning { message: "user_not_found" }),
        Err(_)      => return Response::InternalServerError().json(Warning { message: "db_error" }),
    };

    // Verifikasi password dengan bcrypt
    let password_match = verify(password, &user.password).unwrap_or(false);
    if !password_match {
        return Response::Unauthorized().json(Warning { message: "password_not_match" });
    }

    // Simpan user_id ke session — ini yang dibaca SessionGuard
    if session.insert("user_id", user.id).is_err() {
        return Response::InternalServerError().json(Warning { message: "session_error" });
    }

    Response::Ok().json(Output {
        id: user.id,
        email: user.email,
    })
}

pub async fn register(pool: Pool, data: Json<Logindata>, session: Session) -> Response {

    let email    = data.email.trim();
    let password = &data.password;

    // Cek input kosong
    if email.is_empty() || password.is_empty() {
        return Response::Forbidden().json(Warning { message: "blank_input" });
    }

    let conn = pool.get_ref();

    // Cek apakah email sudah terdaftar
    let existing = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ? LIMIT 1"
    )
    .bind(email)
    .fetch_optional(conn)
    .await;

    match existing {
        Ok(Some(_)) => return Response::Conflict().json(Warning { message: "email_already_used" }),
        Ok(None)    => {},
        Err(_)      => return Response::InternalServerError().json(Warning { message: "db_error" }),
    }

    // Hash password sebelum disimpan
    let hashed = match hash(password, DEFAULT_COST) {
        Ok(h)  => h,
        Err(_) => return Response::InternalServerError().json(Warning { message: "hash_error" }),
    };

    // Insert user baru
    let inserted = sqlx::query(
        "INSERT INTO users (email, password) VALUES (?, ?)"
    )
    .bind(email)
    .bind(&hashed)
    .execute(conn)
    .await;

    let inserted = match inserted {
        Ok(r)  => r,
        Err(_) => return Response::InternalServerError().json(Warning { message: "db_error" }),
    };

    let new_id = inserted.last_insert_id() as Int;

    // Langsung login setelah register
    if session.insert("user_id", new_id).is_err() {
        return Response::InternalServerError().json(Warning { message: "session_error" });
    }

    Response::Created().json(Output {
        id: new_id,
        email: email.to_string(),
    })
}

pub async fn logout(session: Session) -> Response {
    session.purge();
    Response::Ok().json(Warning { message: "logged_out" })
}