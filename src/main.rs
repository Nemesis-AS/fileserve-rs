mod extractors;
mod models;
mod middlewares;
mod routes;
mod utils;

use actix_web::{App, HttpServer, web};
use rust_embed::Embed;
use sqlx::SqlitePool;

use crate::routes::register;

#[derive(Embed)]
#[folder = "static/"]
pub struct Asset;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = 8112;
    let pool = SqlitePool::connect("sqlite:data/db.sqlite3")
        .await
        .expect("Failed to connect to db!");

    println!("Started server at PORT {}!", port);
    HttpServer::new(move || {
        App::new()
            .configure(register)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
