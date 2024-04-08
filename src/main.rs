#[macro_use]
extern crate sqlx;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod middleware;
pub mod handler;
pub mod web;
pub mod appstate;
pub mod socketsession;

use actix_web::{ web::{ get, post, route, Data }, App, HttpServer, cookie::Key, cookie::SameSite };
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::middleware::auth::bearer_validator;
use crate::socketsession::{ Usession, UsessionContainer };
use actix_session::{ Session, SessionMiddleware, storage::CookieSessionStore };
use actix_cors::Cors;

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
        // session
        let appnew = appnew.wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                .cookie_same_site(SameSite::None)
                .build()
        );
        // cors
        let appnew = appnew.wrap(Cors::permissive());
        // database pool
        let appnew = appnew.app_data(Data::new(pool.clone()));
        // load config
        let appnew = appnew.app_data(Data::new(appstate::new()));
        let appnew = appnew.app_data(Data::new(UsessionContainer::new()));
        let appnew = appnew.route("/ws", get().to(handler::websocket::ws));
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
