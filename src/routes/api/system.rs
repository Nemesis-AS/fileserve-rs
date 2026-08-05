use std::time::Duration;

use actix_web::{
    HttpResponse, Responder,
    web::{self, ServiceConfig},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::config::{AppConfig, RestartMode};
use crate::extractors::AuthUser;
use crate::routes::api::types::ApiResponse;
use crate::update::{RestartSignal, UpdateState, UpdateStateData};

use super::users::{reject_if_demo, require_admin};

#[derive(Serialize)]
struct VersionDto {
    version: &'static str,
    target: &'static str,
}

#[derive(Serialize)]
struct UpdateStatusDto {
    current_version: &'static str,
    target: &'static str,
    enabled: bool,
    #[serde(flatten)]
    state: UpdateStateData,
}

impl UpdateStatusDto {
    fn new(state: UpdateStateData, config: &AppConfig) -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION"),
            target: env!("BUILD_TARGET"),
            enabled: config.self_update_enabled,
            state,
        }
    }
}

/// Not admin-only: the config page polls this while waiting for a restart. Not
/// anonymous either — an exact version is free targeting information.
async fn get_version(_auth: AuthUser) -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::ok(
        "Version retrieved",
        VersionDto {
            version: env!("CARGO_PKG_VERSION"),
            target: env!("BUILD_TARGET"),
        },
    ))
}

async fn get_update_status(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    state: web::Data<UpdateState>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    HttpResponse::Ok().json(ApiResponse::ok(
        "Update status retrieved",
        UpdateStatusDto::new(state.snapshot(), &config),
    ))
}

async fn check_for_update(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    state: web::Data<UpdateState>,
) -> impl Responder {
    if let Err(resp) = reject_if_demo(&config) {
        return resp;
    }
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    if !config.self_update_enabled {
        return HttpResponse::ServiceUnavailable()
            .json(ApiResponse::error("Self-update is disabled on this server"));
    }

    let worker_state = state.clone();
    let worker_config = config.get_ref().clone();
    if web::block(move || crate::update::run_check(&worker_state, &worker_config))
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::error("The update check failed to run"));
    }

    HttpResponse::Ok().json(ApiResponse::ok(
        "Update check complete",
        UpdateStatusDto::new(state.snapshot(), &config),
    ))
}

#[derive(Deserialize)]
struct InstallBody {
    version: String,
}

/// The body names the version the admin was looking at, so a release that
/// landed since the page rendered is refused rather than quietly installed.
async fn install_update(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    state: web::Data<UpdateState>,
    body: web::Json<InstallBody>,
) -> impl Responder {
    if let Err(resp) = reject_if_demo(&config) {
        return resp;
    }
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    if !config.self_update_enabled {
        return HttpResponse::ServiceUnavailable()
            .json(ApiResponse::error("Self-update is disabled on this server"));
    }

    let release = match state.try_begin_install(&body.version) {
        Ok(release) => release,
        Err(rejected) => {
            return HttpResponse::Conflict().json(ApiResponse::error(rejected.message()));
        }
    };

    crate::update::spawn_install(state.clone(), config.get_ref().clone(), release);

    HttpResponse::Accepted().json(ApiResponse::ok(
        "Installing update",
        state.install_snapshot(),
    ))
}

async fn get_install_progress(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    state: web::Data<UpdateState>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    HttpResponse::Ok().json(ApiResponse::ok(
        "Install progress retrieved",
        state.install_snapshot(),
    ))
}

#[derive(Serialize)]
struct RestartDto {
    restart_mode: RestartMode,
}

/// Responds before shutting down so the browser gets an answer. The stop is
/// graceful — in-flight uploads finish first.
async fn restart_server(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    restart: web::Data<RestartSignal>,
) -> impl Responder {
    if let Err(resp) = reject_if_demo(&config) {
        return resp;
    }
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    let Some(handle) = restart.handle() else {
        return HttpResponse::ServiceUnavailable()
            .json(ApiResponse::error("The server is not ready to restart yet"));
    };

    restart.request();

    actix_web::rt::spawn(async move {
        actix_web::rt::time::sleep(Duration::from_millis(400)).await;
        handle.stop(true).await;
    });

    HttpResponse::Ok().json(ApiResponse::ok(
        "Restarting",
        RestartDto {
            restart_mode: config.restart_mode.effective(),
        },
    ))
}

pub fn register(config: &mut ServiceConfig) {
    config
        .route("/version", web::get().to(get_version))
        .route("/update", web::get().to(get_update_status))
        .route("/update/check", web::post().to(check_for_update))
        .route("/update/install", web::post().to(install_update))
        .route("/update/progress", web::get().to(get_install_progress))
        .route("/restart", web::post().to(restart_server));
}
