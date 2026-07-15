use actix_web::{
    HttpRequest, HttpResponse, Responder, guard,
    http::header::{ContentLength, ContentType},
    middleware::from_fn,
    web::{self, ServiceConfig},
};
// use serde::Deserialize;
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

use crate::{
    middlewares::tus_resumable,
    routes::api::types::ApiResponse,
    utils::tus::{UploadMetadataFields, decode_metadata},
};

// TUS Spec
async fn get_server_config() -> impl Responder {
    HttpResponse::NoContent()
        .append_header(("Tus-Resumable", "1.0.0"))
        .append_header(("Tus-Version", "1.0.0"))
        .append_header(("Tus-Max-Size", 1073741824))
        .append_header(("Tus-Extension", "creation"))
        .finish()
}

async fn create_upload(req: HttpRequest, pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    const TUS_MAX_SIZE: i64 = 5 * 1024 * 1024 * 1024; // 5 GiB, @todo! Move to global config

    let upload_length_header = req
        .headers()
        .get("Upload-Length")
        .and_then(|v| v.to_str().ok());

    let upload_defer_length_header = req
        .headers()
        .get("Upload-Defer-Length")
        .and_then(|v| v.to_str().ok());

    if upload_length_header.is_some() && upload_defer_length_header.is_some() {
        return HttpResponse::BadRequest().json(ApiResponse::error(
            "Cannot set both Upload-Length and Upload-Defer-Length",
        ));
    }

    let content_length: Option<i64> = if let Some(defer) = upload_defer_length_header {
        if defer != "1" {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Invalid Upload-Defer-Length value"));
        }
        None
    } else {
        let Some(length_str) = upload_length_header else {
            return HttpResponse::BadRequest().json(ApiResponse::error(
                "Missing Upload-Length or Upload-Defer-Length header",
            ));
        };

        let Ok(length) = length_str.parse::<i64>() else {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Invalid Upload-Length header"));
        };

        if length < 0 {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Upload-Length cannot be negative"));
        }

        if length > TUS_MAX_SIZE {
            return HttpResponse::PayloadTooLarge()
                .append_header(("Tus-Max-Size", TUS_MAX_SIZE.to_string()))
                .json(ApiResponse::error("Upload-Length exceeds Tus-Max-Size"));
        }

        Some(length)
    };

    let Some(upload_metadata) = req.headers().get("Upload-Metadata") else {
        return HttpResponse::BadRequest().json(ApiResponse::error("Missing Upload-Metadata"));
    };

    let Ok(meta) = upload_metadata.to_str() else {
        return HttpResponse::BadRequest().json(ApiResponse::error("Invalid metadata"));
    };

    let Ok(metadata) = decode_metadata(meta).and_then(UploadMetadataFields::try_from) else {
        return HttpResponse::BadRequest().json(ApiResponse::error("Invalid metadata"));
    };

    // @todo! Check if file exists first

    let uid = Uuid::new_v4().simple().to_string();
    let expires_at = chrono::Local::now() + chrono::Duration::minutes(60);

    let insert_result = sqlx::query(
        "INSERT INTO uploads(id, file_dir, file_size, file_name, checksum, mime_type, owner_uname, expires_at) \
         VALUES(?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&uid)
    .bind(&metadata.file_dir)
    .bind(content_length)
    .bind(&metadata.file_name)
    .bind(&metadata.checksum)
    .bind(&metadata.mime_type)
    .bind("user")
    .bind(expires_at)
    .fetch_one(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
    }

    HttpResponse::Created()
        .append_header(("Location", format!("/upload/{uid}")))
        .finish()
}

async fn get_upload_offset(
    file_id_param: web::Path<String>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    let file_id = file_id_param.into_inner();

    let upload_res = sqlx::query("SELECT offset, file_size FROM uploads WHERE id = ?")
        .bind(file_id)
        .fetch_optional(pool.get_ref())
        .await;

    let upload = match upload_res {
        Ok(Some(upload)) => upload,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::error("Upload not found"));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let offset: i64 = match upload.try_get("offset") {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let file_size: Option<i64> = match upload.try_get("file_size") {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let mut response = HttpResponse::Ok();
    response
        .append_header(("Upload-Offset", offset))
        .append_header(("Cache-Control", "no-store"));

    match file_size {
        Some(size) => {
            response.append_header(("Upload-Length", size));
        }
        None => {
            response.append_header(("Upload-Defer-Length", "1"));
        }
    }

    response.finish()
}

async fn upload_chunk(
    req: HttpRequest,
    content_type: web::Header<ContentType>,
    content_length: web::Header<ContentLength>,
    file_id_param: web::Path<String>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    let file_id = file_id_param.into_inner();

    if content_type.to_string() != "application/offset+octet-stream" {
        return HttpResponse::UnsupportedMediaType().finish();
    }

    let upload_offset_header = req
        .headers()
        .get("Upload-Offset")
        .and_then(|v| v.to_str().ok());

    if upload_offset_header.is_none() {
        return HttpResponse::BadRequest().json(ApiResponse::error("Upload offset not present"));
    }

    let req_upload_offset = match upload_offset_header.unwrap().parse::<i64>() {
        Ok(v) => {
            if v >= 0 {
                v
            } else {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::error("Invalid upload offset"));
            }
        }
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::error("Invalid upload offset"));
        }
    };

    let upload_res = sqlx::query("SELECT * FROM uploads WHERE id = ?")
        .bind(&file_id)
        .fetch_optional(pool.get_ref())
        .await;

    let upload = match upload_res {
        Ok(Some(upload)) => upload,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::error("Upload not found"));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let upload_offset: i64 = match upload.try_get("offset") {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let file_size: Option<i64> = match upload.try_get("file_size") {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let Some(file_size) = file_size else {
        return HttpResponse::Conflict()
            .json(ApiResponse::error("Upload-Length has not been set for this upload"));
    };

    if req_upload_offset != upload_offset {
        return HttpResponse::Conflict().finish();
    }

    if upload_offset >= file_size {
        return HttpResponse::Conflict().json(ApiResponse::error("Upload already completed"));
    }

    let len = content_length.into_inner().0 as i64;

    if len == 0 {
        return HttpResponse::NoContent()
            .append_header(("Upload-Offset", upload_offset))
            .finish();
    }

    if upload_offset + len > file_size {
        return HttpResponse::BadRequest()
            .json(ApiResponse::error("Chunk exceeds declared upload length"));
    }

    // @todo! Save the file to the disk

    let update_res =
        sqlx::query("UPDATE uploads SET offset = offset + ? WHERE id = ? RETURNING offset")
            .bind(len)
            .bind(&file_id)
            .fetch_one(pool.get_ref())
            .await;

    if let Err(e) = update_res {
        return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
    }

    HttpResponse::NoContent()
        .append_header(("Upload-Offset", upload_offset + len))
        .finish()
}

pub fn register(config: &mut ServiceConfig) {
    config.service(
        web::scope("/upload")
            .wrap(from_fn(tus_resumable))
            .route(
                "",
                web::route().guard(guard::Options()).to(get_server_config),
            )
            .route("", web::post().to(create_upload))
            .route("/{upload_id}", web::head().to(get_upload_offset))
            .route("/{upload_id}", web::patch().to(upload_chunk)),
    );
}
