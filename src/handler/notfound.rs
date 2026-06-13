use crate::web::Response;

// Not found page
pub async fn notfound() -> Response {
    return Response::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../page/404.html"));
}
