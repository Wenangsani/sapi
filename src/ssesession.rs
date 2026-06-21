use actix_web::web::Bytes;
use tokio::sync::mpsc;
use std::sync::{ Arc, Mutex };
use uuid::Uuid;

/// Format SSE yang valid: "data: <pesan>\n\n"
pub fn sse_message(data: &str) -> Bytes {
    Bytes::from(format!("data: {}\n\n", data))
}

/// Format SSE dengan event name: "event: <name>\ndata: <pesan>\n\n"
pub fn sse_event(event: &str, data: &str) -> Bytes {
    Bytes::from(format!("event: {}\ndata: {}\n\n", event, data))
}

/// Satu entri client SSE
pub struct SseEntry {
    pub sse_id:  String,
    pub user_id: Option<u32>,
    tx: mpsc::Sender<Bytes>,
}

#[derive(Clone)]
pub struct SseSession {
    inner: Arc<Mutex<Vec<SseEntry>>>,
}

impl SseSession {
    pub fn new() -> Self {
        SseSession {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Daftarkan client baru, kembalikan (sse_id, Receiver)
    pub fn insert(&self, user_id: Option<u32>) -> (String, mpsc::Receiver<Bytes>) {
        let sse_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::channel::<Bytes>(64);

        let mut inner = self.inner.lock().unwrap();
        inner.push(SseEntry { sse_id: sse_id.clone(), user_id, tx });

        println!("[SSE] Connected sse_id={} user_id={:?}. Total: {}", sse_id, user_id, inner.len());

        (sse_id, rx)
    }

    /// Hapus client berdasarkan sse_id
    pub fn remove(&self, sse_id: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.retain(|e| e.sse_id != sse_id);
        println!("[SSE] Disconnected sse_id={}. Total: {}", sse_id, inner.len());
    }

    pub fn client_count(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Broadcast ke SEMUA client aktif
    pub async fn broadcast(&self, msg: String) {
        self.send_filtered(msg, |_| true).await;
    }

    /// Kirim ke SEMUA client milik user tertentu
    pub async fn send_to_user(&self, user_id: u32, msg: String) {
        self.send_filtered(msg, move |e| e.user_id == Some(user_id)).await;
    }

    /// Kirim Bytes mentah (sudah diformat) ke satu client — sync, non-blocking.
    /// Dipakai untuk event "init" yang dikirim segera setelah insert().
    pub fn send_to_sse_sync(&self, sse_id: &str, bytes: Bytes) {
        let inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.iter().find(|e| e.sse_id == sse_id) {
            let _ = entry.tx.try_send(bytes);
        }
    }

    /// Kirim ke SATU client berdasarkan sse_id
    pub async fn send_to_sse(&self, sse_id: &str, msg: String) {
        let sid = sse_id.to_string();
        self.send_filtered(msg, move |e| e.sse_id == sid).await;
    }

    /// Internal: kumpulkan sender yang cocok predicate di luar lock,
    /// lalu kirim tanpa memegang Mutex.
    async fn send_filtered<F>(&self, msg: String, predicate: F)
    where
        F: Fn(&SseEntry) -> bool,
    {
        let bytes = sse_message(&msg);

        // Kumpulkan (index, sender clone) sambil pegang lock sebentar,
        // lalu lepas lock sebelum send — hindari deadlock & Mutex across await.
        let senders: Vec<(usize, mpsc::Sender<Bytes>)> = {
            let inner = self.inner.lock().unwrap();
            inner.iter()
                .enumerate()
                .filter(|(_, e)| predicate(e))
                .map(|(i, e)| (i, e.tx.clone()))
                .collect()
        };

        // Kirim tanpa lock; catat index yang gagal (receiver sudah drop)
        let mut dead: Vec<usize> = Vec::new();
        for (i, tx) in senders {
            if tx.send(bytes.clone()).await.is_err() {
                dead.push(i);
            }
        }

        // Hapus client mati — proses dari belakang agar index tidak bergeser
        if !dead.is_empty() {
            let mut inner = self.inner.lock().unwrap();
            for i in dead.into_iter().rev() {
                if i < inner.len() {
                    let removed = inner.swap_remove(i);
                    println!("[SSE] Dropping dead client sse_id={}", removed.sse_id);
                }
            }
        }
    }
}