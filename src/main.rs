mod config;
mod extractors;
mod middlewares;
mod models;
mod routes;
mod utils;

use actix_web::{App, HttpServer, web};
use rust_embed::Embed;
use sqlx::SqlitePool;

use crate::config::AppConfig;
use crate::routes::register;

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

    println!("Started server at PORT {}!", port);
    HttpServer::new(move || {
        App::new()
            .configure(register)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
