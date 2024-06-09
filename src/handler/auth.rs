use crate::web::{Pool, Response, Warning};
use crate::web::types::{Int, String, Date};
use crate::web::data::{Json};
use actix_session::Session;
use rand::distributions::{Alphanumeric, DistString};

// Default value if blank
fn default_value() -> String {
    return String::from("");
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
    pub token: String,
}

pub async fn login(pool: Pool, data: Json<Logindata>, session: Session) -> Response {

    let email    = &data.email.trim();
    let password = &data.password;

    // check if input empty
    if email.is_empty() || password.is_empty() {
        return Response::Forbidden().json(Warning {
            message: "blank_input",
        });
    }

    // process search email
    let conn = pool.get_ref();
    let mut recs = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ? LIMIT 1").bind(email).fetch_all(conn).await.unwrap();

    // check if records is empty
    if recs.len() == 0 {
        return Response::Unauthorized().json(Warning {
            message: "user_not_found",
        });
    }

    let mut user = &mut recs[0];

    // check if password match
    if password != &user.password {
        return Response::Unauthorized().json(Warning {
            message: "password_not_match",
        });
    }

    // create token
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    // insert token on session
    session.insert("authtoken", &token);

    // return user data
    return Response::Ok().json(Output {
        id: user.id,
        token: token
    });
}

pub async fn register(pool: Pool, data: Json<Logindata>, session: Session) -> Response {

    let email    = &data.email.trim();
    let password = &data.password;

    // check if input empty
    if email.is_empty() || password.is_empty() {
        return Response::Forbidden().json(Warning {
            message: "blank_input",
        });
    }

    // process search email
    let conn = pool.get_ref();
    let recs = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ? LIMIT 1").bind(email).fetch_all(conn).await.unwrap();

    // check if email already used
    if recs.len() > 0 {
        return Response::Ok().json(Warning {
            message: "email_already_used",
        });
    }

    // add user on records
    let recs_add = sqlx::query("INSERT INTO users (email, password) VALUES (?, ?)").bind(email).bind(password).execute(conn).await.unwrap();

    // create token
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    // insert token on session
    session.insert("authtoken", &token);

    // return user data
    return Response::Ok().json(Output {
        id: recs_add.last_insert_id() as Int,
        token: token
    });
}
