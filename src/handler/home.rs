use crate::appstate::Appstate;
use crate::web::{ Request, Response, data::{Data, Path} };
use actix_session::Session;
use actix_web::cookie::{ Cookie, time::Duration };

#[derive(Deserialize)]
pub struct WelcomePath {
    pub name: String,
}

// Home page
pub async fn home() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../page/home.html"))
}

// Test page
pub async fn test(_state: Data<Appstate>) -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../page/test.html"))
}

// Cookie example
pub async fn _old(state: Data<Appstate>) -> Response {
    let cookie = Cookie::build("my_cookie", "naga")
        .secure(true)
        .http_only(true)
        .max_age(Duration::days(1))
        .finish();

    Response::Ok()
        .cookie(cookie)
        .body("Hello World ".to_owned() + &state.appname)
}

pub async fn welcome(path: Path<WelcomePath>, session: Session ) -> Response {

    let user_id = session.get::<i64>("user_id");
    let user_id = user_id.unwrap_or(None);

    println!("User ID: {:?}", user_id);

    let count = session.get::<i32>("count");
    let count = count.unwrap_or(None).unwrap_or(0);

    session.insert("count", count + 1).ok();

    Response::Ok().body(format!(
        "Welcome {} — visit #{}",
        path.name,
        count + 1
    ))
}