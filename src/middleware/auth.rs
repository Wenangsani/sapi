use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::{ HttpMessage, dev::ServiceRequest, Error };

pub async fn bearer_validator(
    mut req: ServiceRequest,
    credentials: Option<BearerAuth>
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // check token
    let token = req.headers_mut().get("authorization");
    println!("{:?}", token);
    /*
    if credentials.token() == "kudanil" {
        // insert data into the request extensions
        let mut extensions = req.extensions_mut();
        extensions.insert("user".to_string());
    }
    */
    return Ok(req);
}
