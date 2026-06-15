use actix_web::web::Bytes;
use tokio::sync::broadcast;
use std::sync::{ Arc, Mutex };

/// Satu channel broadcast untuk semua client SSE
#[derive(Clone)]
pub struct SseSession {
    inner: Arc<SseInner>,
}

struct SseInner {
    /// sender broadcast — clone-nya dibagikan ke setiap subscriber
    tx: broadcast::Sender<String>,
    /// hitung jumlah client aktif (opsional, berguna untuk debug/monitoring)
    client_count: Mutex<usize>,
}

impl SseSession {
    pub fn new() -> Self {
        // kapasitas 64 = buffer maksimal pesan yang belum dibaca per subscriber
        let (tx, _) = broadcast::channel(64);

        SseSession {
            inner: Arc::new(SseInner {
                tx,
                client_count: Mutex::new(0),
            }),
        }
    }

    /// Buat subscriber baru — dipanggil saat client SSE connect
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        let mut count = self.inner.client_count.lock().unwrap();
        *count += 1;
        println!("SSE client connected. Total: {}", count);
        self.inner.tx.subscribe()
    }

    /// Kurangi counter saat client disconnect
    pub fn unsubscribe(&self) {
        let mut count = self.inner.client_count.lock().unwrap();
        *count = count.saturating_sub(1);
        println!("SSE client disconnected. Total: {}", count);
    }

    /// Broadcast pesan ke semua subscriber
    /// Mengembalikan jumlah penerima aktif (0 jika tidak ada yang online)
    pub fn send(&self, msg: String) -> usize {
        match self.inner.tx.send(msg) {
            Ok(n) => n,
            Err(_) => 0, // tidak ada subscriber aktif
        }
    }

    pub fn client_count(&self) -> usize {
        *self.inner.client_count.lock().unwrap()
    }
}

/// Format SSE yang valid: "data: <pesan>\n\n"
pub fn sse_message(data: &str) -> Bytes {
    Bytes::from(format!("data: {}\n\n", data))
}

/// Format SSE dengan event name: "event: <name>\ndata: <pesan>\n\n"
pub fn sse_event(event: &str, data: &str) -> Bytes {
    Bytes::from(format!("event: {}\ndata: {}\n\n", event, data))
}