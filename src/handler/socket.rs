use crate::web::{Request, Response, Error};

use actix::{AsyncContext, Actor, StreamHandler};
use actix_web_actors::ws::{WebsocketContext, Message, ProtocolError};
use uuid::Uuid;

// Define HTTP actor
#[derive(Clone, Debug)]
pub struct WsConn {
    id: Uuid
}

impl WsConn {

    // Create new socket connection
    fn new() -> WsConn {

        return WsConn {
            id: Uuid::new_v4()
        };
    }

    fn send(&mut self) {

        println!("self: {:?}", self);

        // ctx.text("taa".to_owned());
    }
}

impl Actor for WsConn {

    type Context = WebsocketContext<Self>;
}

// Handler for ws::Message message
impl StreamHandler<Result<Message, ProtocolError>> for WsConn {

    // On socket connect
    fn started(&mut self, ctx: &mut Self::Context) {

        let socket = self;

        println!("socket connect: {}", socket.id);

        let mut socketlist: Vec<WsConn> = Vec::new();

        socketlist.push(socket.clone());

        socket.send();

        println!("socketlist: {:?}", socketlist);

        let address = ctx.address();

        println!("address: {:?}", address);
    }

    // On socket disconnect
    fn finished(&mut self, _ctx: &mut Self::Context) {

        let socket = self;

        println!("socket disconnect: {}", socket.id);
    }
    
    // Handle socket
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {

        // println!("{:#?}", msg);
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

    // Response to websocket connection
    return actix_web_actors::ws::start(WsConn::new(), &req, stream);
}
