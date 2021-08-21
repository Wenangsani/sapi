use crate::web::{Pool, Response, Warning};
use crate::web::types::{Int, String, Date};
use crate::web::data::{Json};

#[derive(Deserialize)]
pub struct Logindata {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: Int,
    pub email: String,
    pub password: String,
    pub created_at: Date,
}

pub async fn login(pool: Pool, data: Json<Logindata>) -> Response {

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

    // check if result empty
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

    // hide real password
    user.password = String::from("******");

    // return user data
    return Response::Ok().json(user);
}

pub async fn register(pool: Pool, data: Json<Logindata>) -> Response {

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

    // add user
    let recs_add = sqlx::query("INSERT INTO users (email, password) VALUES (?, ?)").bind(email).bind(password).execute(conn).await.unwrap();

    // println!("{:#?}", recs_add.last_insert_id());

    // return user data
    let user = User {
        id: recs_add.last_insert_id() as i32,
        email: email.to_string(),
        password: "******".to_string(),
        created_at: chrono::Utc::now(),
    };
    return Response::Ok().json(user);
}
