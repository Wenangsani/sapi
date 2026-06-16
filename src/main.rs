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
pub mod ssesession;
pub mod module;

use actix_web::{ web::{ get, post, route, scope, Data }, App, HttpServer, cookie::Key, cookie::SameSite };
use crate::socketsession::{ Usession, UsessionInner };
use crate::ssesession::SseSession; 
use actix_session::{ SessionMiddleware, storage::CookieSessionStore };
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    
    let pool = sqlx::MySqlPool::connect("mysql://root:mysql@127.0.0.1:3306/actixweb").await?;

    // Gunakan key tetap di production! Key::generate() berubah tiap restart
    let secret_key = Key::generate();

    let socketlist = Usession::new();
    let sselist    = SseSession::new(); 

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
            .app_data(Data::new(socketlist.clone()))
            .app_data(Data::new(sselist.clone()))

            .configure(module::register_open_routes)

            // ── Public scope ─────────────────────────────
            .service(
                scope("")
                    .route("/ws",            get().to(handler::websocket::ws))
                    .route("/sse",           get().to(handler::sse::sse))
                    .route("/sse/send",      post().to(handler::sse::sse_send))
            )

            // ── Protected scope ──────────────────────────
            .service(
                scope("/gate")
                    .wrap(middleware::session_guard::SessionGuard)
                    .configure(module::register_gate_routes)
            )

            .default_service(route().to(handler::notfound::notfound))
    })
    .bind("127.0.0.1:8080")?
    .run().await?;

    Ok(())
}