#[macro_use]
extern crate sqlx;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
mod macros;


pub mod web;
pub mod appstate;
pub mod middleware;
pub mod handler;
pub mod socketsession;

use actix_web::{ web::{ get, post, route, scope, Data }, App, HttpServer, cookie::Key, cookie::SameSite };
use crate::socketsession::{ Usession, UsessionInner };
use actix_session::{ SessionMiddleware, storage::CookieSessionStore };
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    
    let pool = sqlx::MySqlPool::connect("mysql://root:mysql@127.0.0.1:3306/actixweb").await?;

    // Gunakan key tetap di production! Key::generate() berubah tiap restart
    let secret_key = Key::generate();

    HttpServer::new(move || {

        let session_mw = SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
            .cookie_same_site(SameSite::None)   // Sesuaikan dengan kebutuhan aplikasi
            .cookie_http_only(true)             // JS tidak bisa akses cookie
            .cookie_secure(false)               // true jika HTTPS
            .build();

        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .supports_credentials();     // wajib agar cookie dikirim cross-origin

        App::new()
            .wrap(cors)
            .wrap(session_mw)

            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(appstate::new()))
            .app_data(Data::new(Usession::new()))

            // ── Public scope ─────────────────────────────
            .service(
                scope("")
                    .route("/",              get().to(handler::home::home))
                    .route("/test",          get().to(handler::home::test))
                    .route("/auth/login",    post().to(handler::auth::login))
                    .route("/auth/register", post().to(handler::auth::register))
            )

            // ── Protected scope ──────────────────────────
            .service(
                scope("/api")
                    .wrap(middleware::session_guard::SessionGuard)
                    .route("/welcome/{name}", get().to(handler::home::welcome))
                    .route("/ws",             get().to(handler::websocket::ws))
            )

            .default_service(route().to(handler::notfound::notfound))
    })
    .bind("127.0.0.1:8080")?
    .run().await?;

    Ok(())
}