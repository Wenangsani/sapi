use crate::web::{Response, data::{Path}};

#[derive(Deserialize)]
pub struct WelcomePath {
    name: String,
}

// Simple response page
pub async fn home() -> Response {
    return Response::Ok().body("Hello World!");
}

// Response page with path variable
pub async fn welcome(path: Path<WelcomePath>) -> Response {
    return Response::Ok().body(String::from("Welcome ") + &path.name);
}

// Simple Not found page
pub async fn notfound() -> Response {
    return Response::Ok().body("Page not found.");
}