use crate::web::Response;

// Simple Not found page
pub async fn notfound() -> Response {
    let html = include_str!("../page/404.html");
    return Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html);
}
