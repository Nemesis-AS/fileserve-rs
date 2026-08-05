mod auth;
mod files;
mod settings;
mod system;
pub mod types;
mod users;

use actix_web::{
    HttpResponse, Responder,
    web::{self, ServiceConfig, scope},
};
use auth::register as register_auth;
use files::register as register_files;
use settings::register as register_settings;
use system::register as register_system;
use users::{register as register_users, register_account};

use crate::config::AppConfig;
use crate::routes::api::types::ApiResponse;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Status: OK")
}

/// Shape of `GET /config`.
///
/// Everything here is safe to hand an anonymous caller: no storage path, no
/// account counts, no real settings. `maxUploadBytes` is already public via the
/// TUS `OPTIONS` handler, so it is not a new disclosure.
#[derive(serde::Serialize)]
struct PublicConfigDto {
    demo: bool,
    #[serde(rename = "demoTtlMinutes", skip_serializing_if = "Option::is_none")]
    demo_ttl_minutes: Option<u64>,
    #[serde(rename = "demoQuotaBytes", skip_serializing_if = "Option::is_none")]
    demo_quota_bytes: Option<i64>,
    #[serde(rename = "demoShareMaxMinutes", skip_serializing_if = "Option::is_none")]
    demo_share_max_minutes: Option<i64>,
    #[serde(rename = "maxUploadBytes")]
    max_upload_bytes: i64,
}

/// `GET /config` — the handful of server facts the SPA needs before anyone has
/// signed in. Unauthenticated by necessity: the login page renders before any
/// session exists and still has to know whether to offer a demo.
async fn public_config(
    config: web::Data<AppConfig>,
    settings: web::Data<crate::models::Settings>,
) -> impl Responder {
    let max_upload_bytes = config.effective_max_upload(settings.tus_max_size());

    let dto = match config.demo.as_ref() {
        Some(demo) => PublicConfigDto {
            demo: true,
            demo_ttl_minutes: Some(demo.user_ttl.as_secs() / 60),
            demo_quota_bytes: Some(demo.quota_bytes),
            demo_share_max_minutes: Some(demo.share_max_minutes),
            max_upload_bytes,
        },
        None => PublicConfigDto {
            demo: false,
            demo_ttl_minutes: None,
            demo_quota_bytes: None,
            demo_share_max_minutes: None,
            max_upload_bytes,
        },
    };

    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(ApiResponse::ok("Server config", dto))
}

pub fn register(config: &mut ServiceConfig, app_config: &AppConfig) {
    config.route("/health", web::get().to(health_check));
    config.route("/config", web::get().to(public_config));
    config.service(
        scope("auth")
            .configure(|cfg| register_auth(cfg, app_config))
            .configure(register_account),
    );
    config.service(scope("users").configure(register_users));
    config.service(scope("settings").configure(register_settings));
    config.service(scope("system").configure(register_system));
    config.service(scope("files").configure(|cfg| register_files(cfg, app_config)));
}
