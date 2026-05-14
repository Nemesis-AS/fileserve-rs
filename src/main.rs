mod routes;
mod utils;

use actix_web::{App, HttpServer};
use rust_embed::Embed;

use crate::routes::register;

#[derive(Embed)]
#[folder = "static/"]
pub struct Asset;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = 8112;

    println!("Started server at PORT {}!", port);
    HttpServer::new(move || App::new().configure(register))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
