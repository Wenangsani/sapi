use crate::appstate::Appstate;
use crate::web::{ Data, Cookie, Session, Response };
use crate::web::from::Path;
use actix_web::cookie::time::Duration;

#[derive(Deserialize)]
pub struct WelcomeData {
    pub name: String,
}

// Home page
pub async fn home() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_home.html"))
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

pub async fn welcome(path: Path<WelcomeData>, session: Session ) -> Response {
    // auth guard — satu baris
    let user_id = auth!(session);

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