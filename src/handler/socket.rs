use crate::web::{Request, Response, Error};

use actix::{Actor, StreamHandler};
use actix_web_actors::ws::{WebsocketContext, Message, ProtocolError, start};

/// Define HTTP actor
struct MyWs;

impl Actor for MyWs {
    type Context = WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for MyWs {

    // On socket connect
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Connected");
    }

    // On socket disconnect
    fn finished(&mut self, _ctx: &mut Self::Context) {
        println!("Disconnected");
    }
    
    // Handle socket
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        println!("{:#?}", msg);
        match msg {
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Text(text)) => {
                return ctx.text(text);
            },
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn main(req: Request, stream: actix_web::web::Payload) -> Result<Response, Error> {
    let resp = start(MyWs {}, &req, stream);
    return resp;
}
