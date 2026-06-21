use crate::web::{ Data, Request, Response, Error };
use crate::web::from::Json;
use actix_session::Session as HttpSession;
use actix_web::web::Bytes;
use futures::Stream;
use tokio_stream::wrappers::ReceiverStream;
use crate::ssesession::{ SseSession, sse_event };

/// Handler GET /sse — client browser subscribe di sini
pub async fn sse(
    _req:         Request,
    sse_list:     Data<SseSession>,
    http_session: HttpSession,
) -> Response {

    let user_id: Option<u32> = http_session.get::<u32>("user_id").unwrap_or(None);
    let (sse_id, rx) = sse_list.insert(user_id);

    // Kirim event "init" langsung ke client berisi sse_id-nya sendiri
    // Gunakan channel yang baru dibuat — kirim sebelum streaming dimulai
    {
        let init_bytes = sse_event("init", &format!(r#"{{"sse_id":"{}"}}"#, sse_id));
        // insert langsung ke tx melalui clone — tx masih bisa diakses via inner
        // Cara paling mudah: kirim lewat SseSession karena sse_id sudah terdaftar
        sse_list.send_to_sse_sync(&sse_id, init_bytes);
    }

    let sse_list_clone = sse_list.clone();
    let sse_id_clone   = sse_id.clone();

    let stream = build_stream(rx, sse_list_clone, sse_id_clone);

    Response::Ok()
        .insert_header(("Content-Type",      "text/event-stream"))
        .insert_header(("Cache-Control",     "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream)
}

fn build_stream(
    rx:       tokio::sync::mpsc::Receiver<Bytes>,
    sse_list: Data<SseSession>,
    sse_id:   String,
) -> impl Stream<Item = Result<Bytes, Error>> {
    use futures::stream::StreamExt;

    struct DropGuard {
        sse_list: Data<SseSession>,
        sse_id:   String,
    }
    impl Drop for DropGuard {
        fn drop(&mut self) {
            self.sse_list.remove(&self.sse_id);
        }
    }
    let guard = DropGuard { sse_list, sse_id };

    ReceiverStream::new(rx)
        .map(Ok::<Bytes, Error>)
        .scan(guard, |_guard, item| async move { Some(item) })
}

// ─── Struct pesan ────────────────────────────────────────────────────────────

/// Payload JSON dari client: { "from": "...", "message": "...", "target": {...} }
#[derive(serde::Deserialize)]
pub struct SseMessage {
    #[serde(rename = "_from")]
    pub from:    String,
    pub message: String,
    pub target:  SseTarget,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SseTarget {
    All,
    User(u32),
    Sse(String),
}

/// Handler POST /sse/send
pub async fn sse_send(
    sse_list: Data<SseSession>,
    body:     Json<SseMessage>,
) -> Response {

    // Payload yang dikirim ke client SSE adalah JSON string
    // agar bisa dibaca oleh es.onmessage di frontend
    let payload = match serde_json::to_string(&serde_json::json!({
        "_from":   body.from,
        "message": body.message,
    })) {
        Ok(s)  => s,
        Err(_) => return Response::InternalServerError().finish(),
    };

    let count = sse_list.client_count();

    match &body.target {
        SseTarget::All        => sse_list.broadcast(payload).await,
        SseTarget::User(uid)  => sse_list.send_to_user(*uid, payload).await,
        SseTarget::Sse(sid)   => sse_list.send_to_sse(sid, payload).await,
    }

    Response::Ok().json(serde_json::json!({
        "ok": true,
        "total_clients": count,
    }))
}