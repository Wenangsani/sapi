use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::{ HttpMessage, dev::ServiceRequest, Error };

pub async fn bearer_validator(
    mut req: ServiceRequest,
    credentials: Option<BearerAuth>
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // check token
    let mut is_user: bool = false;

    let token = req.headers_mut().get("authorization");

    if token.is_none() == false {
        is_user = true;
    }

    println!("{:?}", token);
    println!("{:?}", is_user);
    /*
    if credentials.token() == "kudanil" {
        // insert data into the request extensions
        let mut extensions = req.extensions_mut();
        extensions.insert("user".to_string());
    }
    */
    return Ok(req);
}
