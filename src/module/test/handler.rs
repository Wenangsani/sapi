use crate::appstate::Appstate;
use crate::web::{ Cookie, Session, Request, Response };
use crate::web::from::{Data, Path};
use actix_web::cookie::time::Duration;


// Test API page
pub async fn api() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_api.html"))
}

// Test Socket page
pub async fn socket() -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_socket.html"))
}

// Test SSE page
pub async fn sse(_req: Request) -> Response {
    Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("page_sse.html"))
}