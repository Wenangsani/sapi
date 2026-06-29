use actix_ws::Session;
use futures::stream::{ FuturesUnordered, StreamExt };
use std::sync::{ Arc, Mutex };
use uuid::Uuid;

/// Satu entri koneksi WebSocket
pub struct SocketEntry {
    pub socket_id: String,
    pub user_id:   Option<u32>,
    pub session:   Session,
}

#[derive(Clone)]
pub struct SocketSession {
    inner: Arc<Mutex<Vec<SocketEntry>>>,
}

impl SocketSession {
    pub fn new() -> Self {
        SocketSession {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Daftarkan koneksi baru, kembalikan socket_id yang di-generate
    pub fn insert(&self, session: Session, user_id: Option<u32>) -> String {
        let socket_id = Uuid::new_v4().to_string();
        let mut inner = self.inner.lock().unwrap();
        inner.push(SocketEntry { socket_id: socket_id.clone(), user_id, session });
        socket_id
    }

    /// Hapus koneksi berdasarkan socket_id
    pub fn remove(&self, socket_id: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.retain(|e| e.socket_id != socket_id);
    }

    /// Jumlah koneksi WebSocket yang aktif saat ini
    pub fn client_count(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Kirim event "init" ke client segera setelah connect — berisi socket_id-nya sendiri
    pub async fn send_init(&self, socket_id: &str) {
        let msg = serde_json::json!({
            "event":     "init",
            "socket_id": socket_id,
        }).to_string();
        self.send_to_socket(socket_id, msg).await;
    }

    /// Broadcast ke SEMUA koneksi aktif
    pub async fn broadcast(&self, msg: String) {
        self.send_filtered(msg, |_| true).await;
    }

    /// Kirim ke SEMUA socket milik user tertentu
    pub async fn send_to_user(&self, user_id: u32, msg: String) {
        self.send_filtered(msg, move |e| e.user_id == Some(user_id)).await;
    }

    /// Kirim ke SATU socket berdasarkan socket_id
    pub async fn send_to_socket(&self, socket_id: &str, msg: String) {
        let sid = socket_id.to_string();
        self.send_filtered(msg, move |e| e.socket_id == sid).await;
    }

    /// Internal: drain entries yang cocok predicate, kirim concurrent, kembalikan yang hidup.
    /// Lock dilepas sebelum await — tidak ada Mutex across await.
    async fn send_filtered<F>(&self, msg: String, predicate: F)
    where
        F: Fn(&SocketEntry) -> bool + Send + 'static,
    {
        // Pisahkan sambil pegang lock sebentar
        let (to_send, mut keep): (Vec<SocketEntry>, Vec<SocketEntry>) = {
            let mut inner = self.inner.lock().unwrap();
            let all: Vec<SocketEntry> = inner.drain(..).collect();
            all.into_iter().partition(|e| predicate(e))
        };

        // Kirim concurrent di luar lock
        let mut unordered = FuturesUnordered::new();
        for mut entry in to_send {
            let msg = msg.clone();
            unordered.push(async move {
                let res = entry.session.text(msg).await;
                res.map(|_| (entry.socket_id, entry.user_id, entry.session))
                   .map_err(|_| eprintln!("[WS] Dropping dead session"))
            });
        }

        while let Some(res) = unordered.next().await {
            if let Ok((socket_id, user_id, session)) = res {
                keep.push(SocketEntry { socket_id, user_id, session });
            }
        }

        // Kembalikan semua yang masih hidup
        let mut inner = self.inner.lock().unwrap();
        inner.extend(keep);
    }
}