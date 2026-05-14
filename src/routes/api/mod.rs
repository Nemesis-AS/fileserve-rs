use actix_web::{HttpResponse, Responder, web};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Status: OK")
}

pub fn register(config: &mut actix_web::web::ServiceConfig) {
    config.route("/health", web::get().to(health_check));
}
