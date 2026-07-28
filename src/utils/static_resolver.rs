use crate::Asset;
use actix_web::HttpResponse;
use mime_guess::from_path;

pub fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

pub fn get_embedded_file_or(path: &str, default: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => match Asset::get(default) {
            Some(content) => HttpResponse::Ok()
                .content_type(from_path(default).first_or_octet_stream().as_ref())
                .body(content.data.into_owned()),
            None => HttpResponse::NotFound().body("404 Not Found"),
        },
    }
}
