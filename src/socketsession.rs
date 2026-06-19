use actix_ws::Session;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::{ Arc, Mutex };

#[derive(Clone)]
pub struct SocketSession {
    inner: Arc<Mutex<SocketInner>>,
}

pub struct SocketInner {
    sessions: Vec<Session>,
}

impl SocketSession {
    pub fn new() -> Self {
        SocketSession {
            inner: Arc::new(
                Mutex::new(SocketInner {
                    sessions: Vec::new(),
                })
            ),
        }
    }

    pub async fn insert(&self, session: Session) {
        let mut inner = self.inner.lock().unwrap();
        inner.sessions.push(session);
    }

    pub async fn send(&self, msg: String) {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => {
                // Handle poison error
                eprintln!("Mutex is poisoned");
                return;
            }
        };

        let mut unordered = FuturesUnordered::new();

        for mut session in inner.sessions.drain(..) {
            let msg = msg.clone();
            unordered.push(async move {
                let res = session.text(msg).await;
                res.map(|_| session).map_err(|_| eprintln!("Dropping session"))
            });
        }

        while let Some(res) = unordered.next().await {
            if let Ok(session) = res {
                inner.sessions.push(session);
            }
        }
    }
}
