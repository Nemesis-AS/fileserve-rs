mod auth;
mod files;
mod settings;
pub mod types;
mod users;

use actix_web::{
    HttpResponse, Responder,
    web::{self, ServiceConfig, scope},
};
use auth::register as register_auth;
use files::register as register_files;
use settings::register as register_settings;
use users::{register as register_users, register_account};

use crate::config::AppConfig;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Status: OK")
}

pub fn register(config: &mut ServiceConfig, app_config: &AppConfig) {
    config.route("/health", web::get().to(health_check));
    config.service(
        scope("auth")
            .configure(register_auth)
            .configure(register_account),
    );
    config.service(scope("users").configure(register_users));
    config.service(scope("settings").configure(register_settings));
    config.service(scope("files").configure(|cfg| register_files(cfg, app_config)));
}
