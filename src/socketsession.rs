use actix_ws::Session;
use std::sync::{Arc, Mutex};

// Struct representing an object
pub struct Usession {
    pub id: i32,
    pub session: Session,
}

pub struct UsessionContainer {
    pub items: Arc<Mutex<Vec<Usession>>>,
}

impl UsessionContainer {

    pub fn new() -> Self {
        UsessionContainer { 
            items: Arc::new(Mutex::new(Vec::new())) 
        }
    }

    pub fn add_session(&self, session: Usession) {
        self.items.lock().unwrap().push(session);
    }
    /*

    pub fn add_session(&mut self, session: Usession) {
        self.items.push(session);
    }

    pub fn delete_session(&mut self, session_id: i32) {
        self.items.retain(|s| s.id != session_id);
    }
    */
}
