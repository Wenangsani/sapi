use crate::socketsession::UsessionContainer;

pub struct Appstate {
    appname: String,
    usession: UsessionContainer
}

pub fn new() -> Appstate {
    return Appstate {
        appname: String::from("sapi"),
        usession: UsessionContainer::new()
    };
}