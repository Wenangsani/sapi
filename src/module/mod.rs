
// daftarkan module di sini
pub mod auth;
pub mod home;

use actix_web::web::ServiceConfig;

// daftarkan route module di sini
pub fn register_open_routes(config: &mut ServiceConfig) {
    auth::open_routes(config);
    home::open_routes(config);
}

// daftarkan route yang butuh auth di sini
pub fn register_gate_routes(config: &mut ServiceConfig) {
    home::gate_routes(config);
}