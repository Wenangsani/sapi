use actix_ws::Session;
use std::sync::{Arc, Mutex};

// Struct representing an object
pub struct Usession {
    pub id: i32,
    // pub session: Session,
}

pub struct UsessionContainer {
    pub count: Arc<Mutex<i32>>,
    pub items: Vec<Usession>,
}

impl UsessionContainer {

    pub fn new() -> Self {
        return UsessionContainer {
            count: Arc::new(Mutex::new(0)),
            items: Vec::new()
        };
    }

    pub fn add_session(&mut self, session: Usession) {
        self.items.push(session);
    }

    /*

    pub fn get_session(&self) -> Vec<Usession> {
        return self.items;
    }
    */

    /*

    pub fn add_session(&mut self, session: Usession) {
        self.items.push(session);
    }

    pub fn delete_session(&mut self, session_id: i32) {
        self.items.retain(|s| s.id != session_id);
    }
    */
}
