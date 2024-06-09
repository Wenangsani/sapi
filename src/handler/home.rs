use crate::web::{Request, Response, data::{Path, Data}, Authdata};
use crate::appstate::Appstate;
use actix_web::cookie::Cookie;
use actix_web::cookie::time::Duration;
use actix_session::Session;
use actix_web::HttpMessage;

#[derive(Deserialize)]
pub struct WelcomePath {
    name: String,
}

// Simple response page
pub async fn home(state: Data<Appstate>) -> Response {
    // set cookie
    let cookie = Cookie::build("my_cookie", "naga").secure(true).http_only(true).max_age(Duration::days(1)).finish();
    return Response::Ok().cookie(cookie).body("Hello World ".to_owned() + &state.appname);
}

// Response page with path variable
pub async fn welcome(mut req: Request, path: Path<WelcomePath>, session: Session) -> Response {

    // get Authdata from middleware
    if let Some(auth_data) = req.extensions().get::<Authdata>() {
        println!("{:?}", auth_data);
    }

    let mut current = 1;

    let mut count = session.get::<i32>("count");

    if count.as_ref().unwrap().is_none() == false {
        current = count.unwrap().unwrap();
    }

    // insert session
    session.insert("count", current + 1);
    
    return Response::Ok().body(String::from("Welcome ") + &path.name + " " + &current.to_string());
}
