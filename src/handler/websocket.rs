use actix_web::web::Payload;
use actix_ws::{ Message, Session, MessageStream };
use actix_session::Session as HttpSession;
use crate::web::{ Data, Request, Response, Error };
use crate::socketsession::SocketSession;
use futures::stream::StreamExt;

/// Struct pesan masuk dari client WebSocket
/// Format JSON: { "_from": "nama", "message": "...", "target": { "type": "All" } }
///              { "_from": "nama", "message": "...", "target": { "type": "User", "value": 5 } }
///              { "_from": "nama", "message": "...", "target": { "type": "Socket", "value": "uuid" } }
#[derive(serde::Deserialize)]
struct WsIncoming {
    #[serde(rename = "_from")]
    from:    String,
    message: String,
    target:  WsTarget,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "value")]
enum WsTarget {
    All,
    User(u32),
    Socket(String),
}

/// Handler loop untuk satu koneksi WebSocket
pub async fn echo_ws(
    mut session:  Session,
    mut msg_stream: MessageStream,
    socketlist:   Data<SocketSession>,
    socket_id:    String,
) {
    println!("[WS] Connected socket_id={}", socket_id);

    let close_reason = loop {
        match msg_stream.next().await {
            Some(Ok(msg)) => {
                match msg {
                    Message::Text(text) => {
                        let text = text.to_string();
                        println!("[WS] recv socket_id={}: {}", socket_id, text);

                        let incoming: WsIncoming = match serde_json::from_str(&text) {
                            Ok(v)  => v,
                            Err(e) => {
                                eprintln!("[WS] JSON parse error: {e}");
                                // Kirim error balik ke pengirim
                                let _ = session.text(serde_json::json!({
                                    "error": "invalid_json",
                                    "detail": e.to_string(),
                                }).to_string()).await;
                                continue;
                            }
                        };

                        // Payload yang diteruskan ke client lain — format JSON
                        let payload = serde_json::json!({
                            "_from":   incoming.from,
                            "message": incoming.message,
                        }).to_string();

                        match incoming.target {
                            WsTarget::All          => socketlist.broadcast(payload).await,
                            WsTarget::User(uid)    => socketlist.send_to_user(uid, payload).await,
                            WsTarget::Socket(sid)  => socketlist.send_to_socket(&sid, payload).await,
                        }
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

                    Message::Pong(_) | Message::Nop => {}

                    Message::Continuation(_) => {
                        eprintln!("[WS] no support for continuation frames");
                    }
                }
            }

            _ => break None,
        }
    };

    socketlist.remove(&socket_id);
    let _ = session.close(close_reason).await;
    println!("[WS] Disconnected socket_id={}", socket_id);
}

/// Entry point HTTP → upgrade ke WebSocket
pub async fn ws(
    req:          Request,
    body:         Payload,
    socketlist:   Data<SocketSession>,
    http_session: HttpSession,
) -> Result<Response, Error> {

    let user_id: Option<u32> = http_session.get::<u32>("user_id").unwrap_or(None);

    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    let socket_id = socketlist.insert(session.clone(), user_id);
    println!("[WS] Registered socket_id={} user_id={:?}", socket_id, user_id);

    // Kirim event init ke client berisi socket_id-nya sendiri
    socketlist.send_init(&socket_id).await;

    actix_rt::spawn(echo_ws(session, msg_stream, socketlist, socket_id));

    Ok(response)
}