#[macro_use]
extern crate sqlx;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

// Custom macros
#[macro_use]
mod macros;

// Modules
pub mod web;
pub mod appstate;
pub mod middleware;
pub mod handler;
pub mod socketsession;
pub mod ssesession;
pub mod module;

// Imports
use actix_web::{ web::{ get, post, route, scope, Data }, App, HttpServer, cookie::Key, cookie::SameSite };
use actix_session::{ SessionMiddleware, storage::CookieSessionStore };
use actix_governor::{Governor, GovernorConfigBuilder};
use crate::socketsession::{ Usession, UsessionInner };
use crate::ssesession::SseSession; 
use actix_cors::Cors;
use dotenvy::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {

    // Load the .env file
    dotenv().ok();

    // Environment variables
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "mysql://dbuser:dbpassword@127.0.0.1:3306/dbname".into());
    let secret_str = env::var("KEY").unwrap_or_else(|_| "secret-key-default-at-least-64-bytes-long-abcdefghijklmnopqrstuvwxyz1234567890".to_string());

    // DataBase connection pool
    let pool = sqlx::MySqlPool::connect(&database_url).await?;

    // Socket session management
    let socketlist = Usession::new();

    // SSE session management
    let sselist    = SseSession::new();

    // Rate limiter Allow 100 requests per 10-second window per peer
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(100)
        .finish()
        .unwrap();

    // Secret key for session middleware
    let secret_key = Key::from(secret_str.as_bytes());

    HttpServer::new(move || {

        // Session middleware configuration
        let session_mw = SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
            .cookie_same_site(SameSite::Lax)    // Sesuaikan dengan kebutuhan aplikasi
            .cookie_http_only(true)             // JS tidak bisa akses cookie
            .cookie_secure(false)               // true jika HTTPS
            .build();

        // CORS configuration
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .supports_credentials();     // wajib agar cookie dikirim cross-origin

        App::new()
            // Attach the rate governor as middleware
            .wrap(Governor::new(&governor_conf))
            .wrap(cors)
            .wrap(session_mw)

            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(appstate::new()))
            .app_data(Data::new(socketlist.clone()))
            .app_data(Data::new(sselist.clone()))

            // ── Public scope ─────────────────────────────
            .service(
                scope("/conn")
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

            // ── Public scope ─────────────────────────────
            .configure(module::register_open_routes)

            .default_service(route().to(handler::notfound::notfound))
    })
    .bind((host, port))?
    .run().await?;

    Ok(())
}