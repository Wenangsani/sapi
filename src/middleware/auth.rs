use crate::web::{Authdata};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::{ HttpMessage, dev::ServiceRequest, Error };
use actix_session::Session;

pub async fn bearer_validator(mut req: ServiceRequest, credentials: Option<BearerAuth>) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    
    // check token
    let mut is_user: bool = false;
    let mut is_cook: bool = false;

    // let token = req.headers_mut().get("authorization");

    if let Some(credentials) = credentials {
        
        // get received token
        let token = credentials.token();

        // Extract the session from the request
        let session = req.extract::<Session>().await.unwrap();

        // get saved token
        let mut realtoken = session.get::<String>("authtoken").unwrap();


        if let Some(realtoken) = realtoken {

            // check token match
            if realtoken == token {
                is_user = true;
            }
        }


    } else {
        // Handle the case where the credentials are not provided
        println!("No token provided");
    }

    // store Authdata in extensions
    req.extensions_mut().insert(Authdata { is_user: is_user });

    println!("Is user: {}", is_user);

    return Ok(req);
}
