use crate::web::{ Request, Response };


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