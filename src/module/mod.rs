
// Daftarkan module di sini
pub mod auth;
pub mod test;
pub mod forum;
pub mod quiz;
pub mod feed;
pub mod messages;
pub mod superadmin;
pub mod home;


use actix_web::web::ServiceConfig;

// Daftarkan route module di sini
pub fn register_open_routes(config: &mut ServiceConfig) {
    auth::open_routes(config);
    forum::open_routes(config);
    test::open_routes(config);
    quiz::open_routes(config);
    feed::open_routes(config);
    messages::open_routes(config);
    superadmin::open_routes(config);
    home::open_routes(config); // ← scope("") selalu paling bawah
}

// Daftarkan route yang butuh auth di sini
pub fn register_gate_routes(config: &mut ServiceConfig) {
    forum::gate_routes(config);
    quiz::gate_routes(config);
    feed::gate_routes(config);
    messages::gate_routes(config);
    superadmin::gate_routes(config);
    home::gate_routes(config);
}