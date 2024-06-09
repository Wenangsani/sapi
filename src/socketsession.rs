use actix_ws::Session;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::{ Arc, Mutex, MutexGuard };

#[derive(Clone)]
pub struct Usession {
    inner: Arc<Mutex<UsessionInner>>,
}

pub struct UsessionInner {
    sessions: Vec<Session>,
}

impl Usession {
    pub fn new() -> Self {
        Usession {
            inner: Arc::new(
                Mutex::new(UsessionInner {
                    sessions: Vec::new(),
                })
            ),
        }
    }

    pub async fn insert(&self, session: Session) {
        let mut inner = self.inner.lock().unwrap();
        inner.sessions.push(session);
    }

    // https://git.asonix.dog/asonix/actix-actorless-websockets/src/branch/main/examples/chat/src/main.rs

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
            eprintln!("chat --- ");
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
