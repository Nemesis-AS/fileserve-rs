mod config;
mod extractors;
mod middlewares;
mod models;
mod routes;
mod utils;

#[cfg(debug_assertions)]
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use rust_embed::Embed;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use crate::config::AppConfig;
use crate::models::{Settings, seed_admin};
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

    let port: u16 = 8112;

    let db_dir = "data";
    std::fs::create_dir_all(db_dir).expect("Failed to create db directory!");

    let connect_options = SqliteConnectOptions::new()
        .filename(format!("{db_dir}/db.sqlite3"))
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(connect_options)
        .await
        .expect("Failed to connect to db!");

        sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations!");

    let (settings, jwt_secret) = Settings::load_or_init(&pool)
        .await
        .expect("Failed to load settings!");
    let config = AppConfig::load(jwt_secret, settings.tus_max_size() as usize);
    let settings = web::Data::new(settings);

    if let Some(admin) = seed_admin(&pool).await.expect("Failed to seed admin!") {
        match admin.generated_password {
            Some(password) => println!(
                "Seeded admin account '{}' with generated password: {}",
                admin.username, password
            ),
            None => println!("Seeded admin account '{}' from ADMIN_PASSWORD.", admin.username),
        }
    }

    let checksum_cache = web::Data::new(ChecksumCache::default());

    println!("Started server at PORT {}!", port);
    HttpServer::new(move || {
        let app = App::new()
            .configure(|cfg| register(cfg, &config))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(settings.clone())
            .app_data(checksum_cache.clone());

        #[cfg(debug_assertions)]
        let app = {
            let mut cors = Cors::default()
                .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
                .allow_any_header()
                .supports_credentials()
                .max_age(3600);
            for origin in &config.allowed_origins {
                cors = cors.allowed_origin(origin);
            }
            app.wrap(cors)
        };

        app
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
