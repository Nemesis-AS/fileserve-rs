pub mod api;
pub mod views;

use actix_web::web::{scope, ServiceConfig};

use api::register as register_api;
use views::register as register_views;

use crate::config::AppConfig;

pub fn register(config: &mut ServiceConfig, app_config: &AppConfig) {
    config.service(scope("/api/v1").configure(|cfg| register_api(cfg, app_config)));
    config.service(scope("").configure(register_views));
}
