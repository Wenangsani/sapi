#[macro_use]
extern crate sqlx;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod web;
pub mod handler;

use actix_web::{web::{post, get}, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let pool = sqlx::MySqlPool::connect("mysql://root:@127.0.0.1:3306/actixweb").await.unwrap();

    HttpServer::new(move || {
        App::new()
        .data(pool.clone())
        .route("/auth/login", post().to(handler::auth::login))
        .route("/auth/register", post().to(handler::auth::register))
        .route("/welcome/{name}", get().to(handler::home::welcome))
        .route("/socket", get().to(handler::socket::main))
        .route("/", get().to(handler::home::home))
        .default_service(actix_web::web::route().to(|| actix_web::HttpResponse::NotFound().body("Page not found.")))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}