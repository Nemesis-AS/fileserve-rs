mod config;
mod extractors;
mod middlewares;
mod models;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use rust_embed::Embed;
use sqlx::SqlitePool;

use crate::config::AppConfig;
use crate::routes::register;
use crate::utils::tus::ChecksumCache;

#[derive(Embed)]
#[folder = "static/"]
pub struct Asset;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(err) = dotenvy::dotenv() {
        println!("An error occurred while loading .env: {}", err.to_string());
    };

    let config: AppConfig = AppConfig::from_env();

    let port: u16 = 8112;
    let pool = SqlitePool::connect("sqlite:data/db.sqlite3")
        .await
        .expect("Failed to connect to db!");
    let checksum_cache = web::Data::new(ChecksumCache::default());

    println!("Started server at PORT {}!", port);
    HttpServer::new(move || {
        // Restrict cross-origin access to the configured allowlist. Credentials
        // are required (the frontend authenticates with cookies), which rules
        // out a wildcard origin — each allowed origin is added explicitly. An
        // empty allowlist (same-origin prod deploy) permits no cross-origin use.
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);
        for origin in &config.allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .configure(|cfg| register(cfg, &config))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(checksum_cache.clone())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
