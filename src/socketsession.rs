use actix_ws::Session;

// Struct representing an object
pub struct Usession {
    id: i32,
    session: Session,
}

pub struct UsessionContainer {
    items: Vec<Usession>,
}

impl UsessionContainer {
    pub fn new() -> Self {
        UsessionContainer { items: Vec::new() }
    }

    pub fn add_session(&mut self, session: Usession) {
        self.items.push(session);
    }

    pub fn delete_session(&mut self, session_id: i32) {
        self.items.retain(|s| s.id != session_id);
    }
}
