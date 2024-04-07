use actix_web::{ middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer };
use actix_ws::{ Message, Session, MessageStream };
use futures_util::{
    future::{self, Either},
    StreamExt as _,
};

pub async fn echo_ws(mut session: actix_ws::Session, mut msg_stream: actix_ws::MessageStream) {
    println!("Connetted");

    let close_reason = loop {
        match msg_stream.next().await {
            Some(Ok(msg)) => {
                println!("msg: {msg:?}");

                match msg {
                    Message::Text(text) => {
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
                };
            }

            // error or end of stream
            _ => break None,
        }
    };

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;

    println!("Disconnetted");
}

// Simple websocket
pub async fn ws(req: HttpRequest, body: web::Payload) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    actix_rt::spawn(echo_ws(session, msg_stream));

    /*
    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {

        }

        let _ = session.close(None).await;
    });
    */

    return Ok(response);
}
