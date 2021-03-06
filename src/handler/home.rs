use crate::web::{Response};
use crate::web::datas::{Path};

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
    return Response::Ok().body("Welcome ".to_owned() + &path.name);
}