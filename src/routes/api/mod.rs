mod auth;
pub mod types;

use actix_web::{
    HttpResponse, Responder,
    web::{self, scope},
};
use auth::register as register_auth;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Status: OK")
}

pub fn register(config: &mut actix_web::web::ServiceConfig) {
    config.route("/health", web::get().to(health_check));
    config.service(scope("auth").configure(register_auth));
}
