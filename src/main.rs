#[macro_use]
extern crate sqlx;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod middleware;
pub mod handler;
pub mod web;
// pub mod config;

use actix_web::{ web::{ get, post, route, Data }, App, HttpServer };
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::middleware::auth::bearer_validator;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // connect to database
    let pool = sqlx::MySqlPool
        ::connect("mysql://root:mysql@127.0.0.1:3306/actixweb").await
        .unwrap();

    // route list
    return HttpServer::new(move || {
        let appnew = App::new();
        // bearer auth
        let appnew = appnew.wrap(HttpAuthentication::with_fn(bearer_validator));
        // database pool
        let appnew = appnew.app_data(Data::new(pool.clone()));
        // load config
        // let appnew = appnew.app_data(config::app());
        let appnew = appnew.route("/auth/login", post().to(handler::auth::login));
        let appnew = appnew.route("/auth/register", post().to(handler::auth::register));
        let appnew = appnew.route("/welcome/{name}", get().to(handler::home::welcome));
        let appnew = appnew.route("/", get().to(handler::home::home));
        let appnew = appnew.default_service(route().to(handler::notfound::notfound));
        return appnew;
    })
        .bind("127.0.0.1:8080")?
        .run().await;
}
