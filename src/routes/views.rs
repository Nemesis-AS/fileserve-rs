use crate::utils::static_resolver::handle_embedded_file;
use actix_web::{
    HttpResponse, Responder, get,
    web::{Path, ServiceConfig},
};

#[get("/")]
pub async fn index() -> HttpResponse {
    handle_embedded_file("index.html")
}

#[get("/{_:.*}")]
pub async fn dist(path: Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

pub fn register(config: &mut ServiceConfig) {
    config.service(index).service(dist);
}
