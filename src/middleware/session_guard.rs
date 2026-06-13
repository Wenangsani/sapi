use actix_web::{
    dev::{ forward_ready, Service, ServiceRequest, ServiceResponse, Transform },
    Error, HttpResponse, body::EitherBody,
};
use actix_session::SessionExt;
use futures_util::future::{ ok, Ready, LocalBoxFuture };
use std::rc::Rc;
use crate::web::{ ApiResponse };
use crate::web::types::Int;

// ── Transform (factory) ──────────────────────────────────────────────────────

pub struct SessionGuard;

impl<S, B> Transform<S, ServiceRequest> for SessionGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response  = ServiceResponse<EitherBody<B>>;
    type Error     = Error;
    type Transform = SessionGuardMiddleware<S>;
    type InitError = ();
    type Future    = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SessionGuardMiddleware { service: Rc::new(service) })
    }
}

// ── Middleware ───────────────────────────────────────────────────────────────

pub struct SessionGuardMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SessionGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error    = Error;
    type Future   = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let session = req.get_session();

            let authenticated = session.get::<Int>("user_id")
                .unwrap_or(None)
                .is_some();

            if authenticated {
                let res = service.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                let response = HttpResponse::Unauthorized().json(ApiResponse {
                    success: false,
                    message: "unauthorized".into(),
                    data: None,
                    meta: None,
                });
                Ok(req.into_response(response).map_into_right_body())
            }
        })
    }
}