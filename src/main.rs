#[macro_use]
extern crate sqlx;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod handler;
pub mod web;
pub mod config;

use actix_web::{
    web::{get, post, route, Data},
    App, HttpResponse, HttpServer,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Connect to database
    let pool = sqlx::MySqlPool::connect("mysql://root:mysql@127.0.0.1:3306/actixweb")
        .await
        .unwrap();

    // Route list
    HttpServer::new(move || {
        let appnew = App::new();
        let appnew = appnew.app_data(Data::new(pool.clone()));
        let appnew = appnew.app_data(config::app());
        let appnew = appnew.route("/auth/login", post().to(handler::auth::login));
        let appnew = appnew.route("/auth/register", post().to(handler::auth::register));
        let appnew = appnew.route("/welcome/{name}", get().to(handler::home::welcome));
        let appnew = appnew.route("/", get().to(handler::home::home));
        let appnew = appnew.default_service(route().to(handler::home::notfound));
        return appnew;
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
