use actix_web::{ web, Error };
use crate::web::{ Request, Response };
use crate::web::from::{ Data, Path, Json };
use actix_web::rt::time::interval;
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use std::time::Duration;

use crate::appstate::Appstate;
use crate::ssesession::{ SseSession, sse_message, sse_event };

/// Handler GET /sse — client browser subscribe di sini
pub async fn sse(
    req: Request,
    state: Data<Appstate>,
    sse_list: Data<SseSession>,
) -> Response {

    // Buat subscriber untuk client ini
    let rx = sse_list.subscribe();

    // Clone untuk dipakai di drop (unsubscribe saat stream selesai)
    let sse_list_clone = sse_list.clone();

    // Ubah broadcast::Receiver menjadi Stream
    let broadcast_stream = BroadcastStream::new(rx)
        .filter_map(|res| async move {
            match res {
                Ok(msg) => Some(Ok::<_, Error>(sse_message(&msg))),
                Err(e) => {
                    eprintln!("SSE stream error: {:?}", e);
                    None // lagged/dropped, skip saja
                }
            }
        });

    // Kirim response SSE
    Response::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no")) // penting untuk Nginx
        .streaming(broadcast_stream)

    // Catatan: unsubscribe (decrement counter) tidak bisa dipanggil di sini
    // karena streaming sudah pindah ke async context.
    // Gunakan client_count() hanya sebagai estimasi, atau pakai Arc<AtomicUsize>.
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SseMessage {
    pub message: String,
    pub _from:   String,  // ← tambahkan ini
}

/// Handler POST /sse/send — endpoint untuk kirim pesan ke semua client
/// Ekuivalen dengan Message::Text di WebSocket handler
pub async fn sse_send(
    sse_list: Data<SseSession>,
    body:     Json<SseMessage>,
) -> Response {
    // Serialize ulang seluruh struct jadi JSON string, lalu broadcast
    let payload = serde_json::to_string(&*body).unwrap_or_default();
    let count   = sse_list.send(payload);

    Response::Ok().json(serde_json::json!({
        "ok": true,
        "delivered_to": count,
    }))
}