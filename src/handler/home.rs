use crate::web::{Request, Response, data::{Path, Data}};
use crate::appstate::Appstate;
use actix_web::cookie::Cookie;
use actix_web::cookie::time::Duration;
use actix_session::Session;
use actix_web::HttpMessage;

#[derive(Deserialize)]
pub struct WelcomePath {
    name: String,
}

// Home page
pub async fn home() -> Response {
    let html = include_str!("../page/home.html");

    return Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html);
}

// Simple response page
pub async fn test(state: Data<Appstate>) -> Response {
    return Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../page/test.html"));

}

// Simple response page
pub async fn _old(state: Data<Appstate>) -> Response {
    // set cookie
    let cookie = Cookie::build("my_cookie", "naga").secure(true).http_only(true).max_age(Duration::days(1)).finish();
    return Response::Ok().cookie(cookie).body("Hello World ".to_owned() + &state.appname);
}

pub async fn welcome(mut req: Request, path: Path<WelcomePath>, session: Session) -> Response {

    // Ambil user_id yang di-set saat login
    let user_id = session.get::<i64>("user_id").unwrap_or(None);
    println!("User ID: {:?}", user_id);

    // Hitung kunjungan — cara lebih ringkas
    let count = session.get::<i32>("count").unwrap_or(None).unwrap_or(0);
    session.insert("count", count + 1).ok();

    Response::Ok().body(format!("Welcome {} — visit #{}", path.name, count + 1))
}
