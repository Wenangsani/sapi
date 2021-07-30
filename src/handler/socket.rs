use crate::web::{WsConn, Request, Response, Error};

use actix::{AsyncContext, Actor, StreamHandler};
use actix_web_actors::ws::{WebsocketContext, Message, ProtocolError};
use uuid::Uuid;


pub struct WsContext<'a> {
    id: Uuid,
    ctx: &'a mut WebsocketContext<WsConn>
}

impl Actor for WsConn {

    type Context = WebsocketContext<Self>;
}

// Handler for ws::Message
impl StreamHandler<Result<Message, ProtocolError>> for WsConn {

    // On socket connect
    fn started(&mut self, ctx: &mut Self::Context) {

        let socket = self;

        println!("socket connect: {}", socket.id);

        /*
        let mut socketlist: Vec<WsConn> = Vec::new();
        socketlist.push(socket.clone());
        println!("socketlist: {:?}", socketlist);
        */

        socket.send();

        ctx.text("ayam kalkun ...");

        let ma = WsContext {
            id: socket.id,
            ctx: ctx,
        };

        // socket.textmessage(ma);

        // let address = ctx.address();

        // println!("address: {:?}", address);
    }

    // On socket disconnect
    fn finished(&mut self, _ctx: &mut Self::Context) {

        let socket = self;

        println!("socket disconnect: {}", socket.id);
    }
    
    // Handle socket message
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {

        println!("{:#?}", msg);
        
        match msg {
            Ok(Message::Ping(msg)) => {

                return ctx.pong(&msg);
            },
            Ok(Message::Text(text)) => {

                return ctx.text(text);
            },
            Ok(Message::Binary(bin)) => {
                
                return ctx.binary(bin);
            },
            _ => (),
        }
    }
}

pub async fn main(req: Request, stream: actix_web::web::Payload) -> Result<Response, Error> {

    // Response to websocket connection
    return actix_web_actors::ws::start(WsConn::new(), &req, stream);
}
