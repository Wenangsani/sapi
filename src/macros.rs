macro_rules! auth {
    ($session:expr) => {
        $session.get::<crate::web::data::UInt>("user_id").unwrap_or(None)
    };
}