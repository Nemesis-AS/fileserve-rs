use actix_web::{
    HttpResponse, Responder,
    web::{self, ServiceConfig},
};
use sqlx::{Pool, Sqlite};

use crate::extractors::AuthUser;
use crate::models::{Settings, SettingsPatch};
use crate::routes::api::types::ApiResponse;
use super::users::require_admin;

/// `GET /settings` — the current server settings. Admin-only; the JWT secret is
/// never part of this shape, so it can't leak here.
async fn get_settings(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    settings: web::Data<Settings>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    HttpResponse::Ok().json(ApiResponse::ok("Settings retrieved", settings.snapshot()))
}

/// `PATCH /settings` — update one or more settings. Admin-only. Persists to the
/// DB and swaps the live copy, so changes take effect without a restart. Note
/// the payload buffering ceiling is fixed at boot, so a raised upload limit is
/// enforced immediately but the coarse per-request buffer bound only widens on
/// restart. Changing the storage path only affects *new* uploads — existing
/// files are not moved.
async fn update_settings(
    auth: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    settings: web::Data<Settings>,
    body: web::Json<SettingsPatch>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    let patch = body.into_inner();

    // Validate before touching the DB so a bad field can't half-apply.
    if let Some(path) = &patch.storage_path {
        if path.trim().is_empty() {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Storage path cannot be empty"));
        }
    }
    if let Some(size) = patch.tus_max_size {
        if size <= 0 {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Max upload size must be positive"));
        }
    }
    if let Some(quota) = patch.default_quota_bytes {
        if quota < 0 {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Default quota cannot be negative"));
        }
    }

    // Normalize the path (trim) before it's persisted.
    let patch = SettingsPatch {
        storage_path: patch.storage_path.map(|p| p.trim().to_string()),
        ..patch
    };

    match settings.update(pool.get_ref(), patch).await {
        Ok(updated) => HttpResponse::Ok().json(ApiResponse::ok("Settings updated", updated)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

pub fn register(config: &mut ServiceConfig) {
    config
        .route("", web::get().to(get_settings))
        .route("", web::patch().to(update_settings));
}
