
// Daftarkan module di sini
pub mod auth;
pub mod test;
pub mod forum;
pub mod home;


use actix_web::web::ServiceConfig;

// Daftarkan route module di sini
pub fn register_open_routes(config: &mut ServiceConfig) {
    auth::open_routes(config);
    forum::open_routes(config);
    test::open_routes(config);
    home::open_routes(config); // ← scope("") selalu paling bawah
}

// Daftarkan route yang butuh auth di sini
pub fn register_gate_routes(config: &mut ServiceConfig) {
    forum::gate_routes(config);
    home::gate_routes(config);
}