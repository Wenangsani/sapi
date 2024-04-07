use crate::socketsession::UsessionContainer;

pub struct Appstate {
    pub appname: String,
    pub usession: UsessionContainer
}

pub fn new() -> Appstate {
    return Appstate {
        appname: String::from("sapi"),
        usession: UsessionContainer::new()
    };
}