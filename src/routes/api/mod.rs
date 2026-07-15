mod auth;
mod files;
pub mod types;

use actix_web::{
    HttpResponse, Responder,
    web::{self, ServiceConfig, scope},
};
use auth::register as register_auth;
use files::register as register_files;

use crate::config::AppConfig;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Status: OK")
}

pub fn register(config: &mut ServiceConfig, app_config: &AppConfig) {
    config.route("/health", web::get().to(health_check));
    config.service(scope("auth").configure(register_auth));
    config.service(scope("files").configure(|cfg| register_files(cfg, app_config)));
}
