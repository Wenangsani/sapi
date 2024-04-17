use actix_web::{ middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer };
use actix_ws::{ Message, Session, MessageStream };
// use futures_util::{ future::{ self, Either }, StreamExt as _ };

use crate::web::data::Data;
use crate::appstate::Appstate;
use crate::socketsession::{ Usession, UsessionInner };
use std::sync::{Arc, Mutex};
use rand::Rng;
use futures::stream::{FuturesUnordered, StreamExt};

pub async fn echo_ws(mut session: Session, mut msg_stream: MessageStream, socketlist: Data<Usession>) {
    println!("Connetted");

    let mut rng = rand::thread_rng();

    let close_reason = loop {
        match msg_stream.next().await {
            Some(Ok(msg)) => {

                println!("msg: {msg:?}");

                match msg {
                    Message::Text(text) => {

                        /*
                        let mut list = socketlist.get_session();

                        for sc in list.iter_mut() {
                            println!("{:?}", sc.id);
                            sc.session.text("dsfsdf").await.unwrap();
                        }
                        */
                        
                        session.text(text).await.unwrap();
                    }

                    Message::Binary(bin) => {
                        session.binary(bin).await.unwrap();
                    }

                    Message::Close(reason) => {
                        break reason;
                    }

                    Message::Ping(bytes) => {
                        let _ = session.pong(&bytes).await;
                    }

                    Message::Pong(_) => {}

                    Message::Continuation(_) => {
                        println!("no support for continuation frames");
                    }

                    // no-op; ignore
                    Message::Nop => {}
                }
            }

            // error or end of stream
            _ => {
                break None;
            }
        }
    };

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;

    /**
    * add delete socketlist here !
    */

    println!("Disconnetted");
}

// Simple websocket
pub async fn ws(req: HttpRequest, body: web::Payload, state: Data<Appstate>, socketlist: Data<Usession>) -> Result<HttpResponse, Error> {

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    socketlist.insert(session.clone()).await;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    actix_rt::spawn(echo_ws(session, msg_stream, socketlist));

    return Ok(response);
}
