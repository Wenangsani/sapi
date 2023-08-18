use crate::web::{Response, data::{Path}};
use actix_web::cookie::Cookie;
use actix_web::cookie::time::Duration;
use actix_session::Session;

#[derive(Deserialize)]
pub struct WelcomePath {
    name: String,
}

// Simple response page
pub async fn home() -> Response {
    // set cookie
    let cookie = Cookie::build("my_cookie", "naga").secure(true).http_only(true).max_age(Duration::days(1)).finish();
    return Response::Ok().cookie(cookie).body("Hello World!");
}

// Response page with path variable
pub async fn welcome(path: Path<WelcomePath>, session: Session) -> Response {

    let mut current = 1;

    let mut count = session.get::<i32>("count");

    if count.as_ref().unwrap().is_none() == false {
        current = count.unwrap().unwrap();
    }

    // insert session
    session.insert("count", current + 1);
    
    return Response::Ok().body(String::from("Welcome ") + &path.name + " " + &current.to_string());
}
