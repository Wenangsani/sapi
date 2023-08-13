use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::{ HttpMessage, dev::ServiceRequest, Error };

pub async fn bearer_validator(
    mut req: ServiceRequest,
    credentials: Option<BearerAuth>
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // check token
    let mut is_user: bool = false;
    let mut is_cook: bool = false;

    let token = req.headers_mut().get("authorization");

    if token.is_none() == false {
        is_user = true;
    }

    // println!("{:?}", token);
    // println!("{:?}", is_user);

    // let x:Cookie = req.cookie("ayam").unwrap();
    let x = req.cookie("my_cookie");

    // println!("{:?}", x);

    /*
    if x == None {
        // is_cook = true;
        println!("{:?}", x);
    }*/

    // println!("======");

    /*
    if credentials.token() == "kudanil" {
        // insert data into the request extensions
        let mut extensions = req.extensions_mut();
        extensions.insert("user".to_string());
    }
    */
    return Ok(req);
}
