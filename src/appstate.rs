pub struct Appstate {
    pub appname: String
}

pub fn new() -> Appstate {
    return Appstate {
        appname: String::from("sapi")
    };
}