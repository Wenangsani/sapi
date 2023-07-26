use actix_web_httpauth::extractors::bearer::{ BearerAuth };
use actix_web::{ HttpMessage, dev::ServiceRequest, Error };

pub async fn bearer_validator(
    req: ServiceRequest,
    credentials: BearerAuth
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // check token
    if credentials.token() == "kudanil" {
        // insert data into the request extensions
        let mut extensions = req.extensions_mut();
        extensions.insert("user".to_string());
    }
    return Ok(req);
}
