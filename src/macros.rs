macro_rules! auth {
    ($session:expr) => {
        match $session.get::<crate::web::data::Int>("user_id").unwrap_or(None) {
            Some(id) => id,
            None => return actix_web::HttpResponse::Unauthorized().json(crate::web::ApiResponse {
                success: false,
                message: "unauthorized".into(),
                data: None,
                meta: None,
            }),
        }
    };
}