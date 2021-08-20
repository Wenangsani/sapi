pub struct AppConfig {
    app_name: String,
}

pub fn app() -> AppConfig {
    return AppConfig {
        app_name: String::from("sapi"),
    };
}