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
        // @todo! Use apt CORS policy
        let cors = Cors::permissive();

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
