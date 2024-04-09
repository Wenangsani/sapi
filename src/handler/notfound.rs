use crate::web::Response;

// Simple Not found page
pub async fn notfound() -> Response {
    return Response::NotFound().body("Page not found.");
}
