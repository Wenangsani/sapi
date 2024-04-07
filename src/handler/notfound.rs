use crate::web::Response;

// Simple Not found page
pub async fn notfound() -> Response {
    return Response::Ok().body("Page not found.");
}
