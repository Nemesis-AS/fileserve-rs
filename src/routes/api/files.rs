use std::{
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
};

use actix_files::NamedFile;
use actix_web::{
    HttpRequest, HttpResponse, Responder, guard,
    http::header::{
        ContentDisposition, ContentLength, ContentType, DispositionParam, DispositionType,
    },
    middleware::from_fn,
    web::{self, ServiceConfig},
};
use blake2::Digest;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header as JwtHeader, encode};
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    extractors::{AuthUser, ShareClaims, verify_share_token},
    middlewares::tus_resumable,
    models::FileRecord,
    routes::api::types::{
        ApiResponse, DownloadQuery, FileSearchQuery, ShareRequestBody, ShareResponse,
    },
    utils::tus::{
        ChecksumCache, UploadMetadataFields, checksum_hex, decode_metadata, ensure_upload_file,
        final_file_path, hasher_from_prefix, upload_file_path,
    },
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

async fn create_upload(
    req: HttpRequest,
    user: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
) -> impl Responder {
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

        if length > config.tus_max_size as i64 {
            return HttpResponse::PayloadTooLarge()
                .append_header(("Tus-Max-Size", config.tus_max_size.to_string()))
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
    .bind(&user.username)
    .bind(expires_at)
    .fetch_one(pool.get_ref())
    .await;

    if let Err(e) = insert_result {
        return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
    }

    if let Some(size) = content_length {
        let path = upload_file_path(&config, &uid);

        let create_result = web::block(move || -> std::io::Result<()> {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            let file = File::create(&path)?;
            file.set_len(size as u64)?;
            Ok(())
        })
        .await;

        if !matches!(create_result, Ok(Ok(()))) {
            let _ = sqlx::query("DELETE FROM uploads WHERE id = ?")
                .bind(&uid)
                .execute(pool.get_ref())
                .await;

            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to allocate upload file"));
        }
    }

    HttpResponse::Created()
        .append_header(("Location", format!("{}/{uid}", req.path())))
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
    body: web::Bytes,
    content_type: web::Header<ContentType>,
    content_length: web::Header<ContentLength>,
    file_id_param: web::Path<String>,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    checksum_cache: web::Data<ChecksumCache>,
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

    let mut file_size: Option<i64> = match upload.try_get("file_size") {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    if file_size.is_none() {
        if let Some(len_header) = req
            .headers()
            .get("Upload-Length")
            .and_then(|v| v.to_str().ok())
        {
            let Ok(declared_size) = len_header.parse::<i64>() else {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::error("Invalid Upload-Length header"));
            };

            if declared_size < 0 {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::error("Upload-Length cannot be negative"));
            }

            if declared_size > config.tus_max_size as i64 {
                return HttpResponse::PayloadTooLarge()
                    .append_header(("Tus-Max-Size", config.tus_max_size.to_string()))
                    .json(ApiResponse::error("Upload-Length exceeds Tus-Max-Size"));
            }

            if declared_size < upload_offset {
                return HttpResponse::BadRequest().json(ApiResponse::error(
                    "Upload-Length is smaller than the current offset",
                ));
            }

            let update_size_res = sqlx::query("UPDATE uploads SET file_size = ? WHERE id = ?")
                .bind(declared_size)
                .bind(&file_id)
                .execute(pool.get_ref())
                .await;

            if let Err(e) = update_size_res {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::error(&e.to_string()));
            }

            file_size = Some(declared_size);
        }
    }

    if req_upload_offset != upload_offset {
        return HttpResponse::Conflict().finish();
    }

    if let Some(size) = file_size {
        if upload_offset >= size {
            return HttpResponse::Conflict().json(ApiResponse::error("Upload already completed"));
        }
    }

    let len = content_length.into_inner().0 as i64;

    if len == 0 {
        return HttpResponse::NoContent()
            .append_header(("Upload-Offset", upload_offset))
            .finish();
    }

    if let Some(size) = file_size {
        if upload_offset + len > size {
            return HttpResponse::BadRequest()
                .json(ApiResponse::error("Chunk exceeds declared upload length"));
        }
    }

    let new_offset = upload_offset + len;
    let upload_path = upload_file_path(&config, &file_id);

    let cached_hasher = checksum_cache.lock().unwrap().get(&file_id).cloned();

    let mut hasher = match cached_hasher {
        Some(hasher) => hasher,
        None => {
            let prefix_path = upload_path.clone();
            match web::block(move || hasher_from_prefix(&prefix_path, upload_offset as u64)).await {
                Ok(Ok(hasher)) => hasher,
                _ => {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::error("Failed to resume checksum state"));
                }
            }
        }
    };

    hasher.update(&body);
    let computed_checksum = checksum_hex(&hasher);

    let write_result = web::block(move || -> std::io::Result<()> {
        let mut file = ensure_upload_file(&upload_path, file_size)?;
        file.seek(SeekFrom::Start(upload_offset as u64))?;
        file.write_all(&body)
    })
    .await;

    if !matches!(write_result, Ok(Ok(()))) {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::error("Failed to write chunk to disk"));
    }

    checksum_cache
        .lock()
        .unwrap()
        .insert(file_id.clone(), hasher);

    let update_res = sqlx::query(
        "UPDATE uploads SET offset = offset + ?, computed_checksum = ? WHERE id = ? RETURNING offset",
    )
    .bind(len)
    .bind(&computed_checksum)
    .bind(&file_id)
    .fetch_one(pool.get_ref())
    .await;

    if let Err(e) = update_res {
        return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
    }

    if let Some(size) = file_size {
        if new_offset >= size {
            checksum_cache.lock().unwrap().remove(&file_id);

            let client_checksum: Option<String> = upload.try_get("checksum").ok();
            if let Some(client_checksum) = &client_checksum {
                if client_checksum != &computed_checksum {
                    eprintln!(
                        "WARN: checksum mismatch for upload {file_id}: client reported {client_checksum}, computed {computed_checksum}"
                    );
                }
            }

            let upload_path = upload_file_path(&config, &file_id);
            let final_path = final_file_path(&config, &computed_checksum);

            let move_result = web::block(move || -> std::io::Result<()> {
                if final_path.exists() {
                    return fs::remove_file(&upload_path);
                }
                if let Some(parent) = final_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::rename(&upload_path, &final_path)
            })
            .await;

            if !matches!(move_result, Ok(Ok(()))) {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::error("Failed to finalize upload"));
            }

            let finalize_res = sqlx::query(
                "UPDATE uploads SET status = 'completed', finished_at = date() WHERE id = ?",
            )
            .bind(&file_id)
            .execute(pool.get_ref())
            .await;

            if let Err(e) = finalize_res {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::error(&e.to_string()));
            }

            let file_dir: String = upload
                .try_get("file_dir")
                .unwrap_or_else(|_| String::from("/"));
            let file_name: String = match upload.try_get("file_name") {
                Ok(v) => v,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::error(&e.to_string()));
                }
            };
            let mime_type: String = match upload.try_get("mime_type") {
                Ok(v) => v,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::error(&e.to_string()));
                }
            };
            let owner_uname: String = match upload.try_get("owner_uname") {
                Ok(v) => v,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::error(&e.to_string()));
                }
            };

            let insert_file_res = sqlx::query(
                "INSERT INTO files(id, file_name, file_dir, mime_type, file_size, checksum, owner_uname) \
                 VALUES(?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().simple().to_string())
            .bind(&file_name)
            .bind(&file_dir)
            .bind(&mime_type)
            .bind(size)
            .bind(&computed_checksum)
            .bind(&owner_uname)
            .execute(pool.get_ref())
            .await;

            if let Err(e) = insert_file_res {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::error(&e.to_string()));
            }
        }
    }

    HttpResponse::NoContent()
        .append_header(("Upload-Offset", new_offset))
        .finish()
}

async fn download_file(
    req: HttpRequest,
    file_id: web::Path<String>,
    query: web::Query<DownloadQuery>,
    user: Option<AuthUser>,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let file_id = file_id.into_inner();

    let file = match sqlx::query_as::<_, FileRecord>(
        "SELECT id, file_name, mime_type, file_size, checksum, owner_uname FROM files WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&file_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(file)) => file,
        Ok(None) => return HttpResponse::NotFound().json(ApiResponse::error("File not found")),
        Err(e) => return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    };

    let owned_by_user = user
        .as_ref()
        .is_some_and(|user| user.username == file.owner_uname);

    let authorized_by_link = query
        .token
        .as_deref()
        .is_some_and(|token| verify_share_token(token, &config.jwt_secret, &file.id));

    if !owned_by_user && !authorized_by_link {
        return HttpResponse::NotFound().json(ApiResponse::error("File not found"));
    }

    let path = final_file_path(&config, &file.checksum);

    let named_file = match NamedFile::open(&path) {
        Ok(named_file) => named_file,
        Err(_) => return HttpResponse::NotFound().json(ApiResponse::error("File not found")),
    };

    let content_type: mime::Mime = file
        .mime_type
        .parse()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

    let named_file = named_file
        .set_content_type(content_type)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(file.file_name.clone())],
        });

    named_file.into_response(&req)
}

async fn create_share_link(
    file_id: web::Path<String>,
    body: web::Json<ShareRequestBody>,
    user: AuthUser,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let file_id = file_id.into_inner();

    let owns_file: Option<String> = match sqlx::query_scalar(
        "SELECT id FROM files WHERE id = ? AND owner_uname = ? AND deleted_at IS NULL",
    )
    .bind(&file_id)
    .bind(&user.username)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    if owns_file.is_none() {
        return HttpResponse::NotFound().json(ApiResponse::error("File not found"));
    }

    let minutes = body.expires_in_minutes.unwrap_or(60).clamp(1, 60 * 24 * 7);
    let expires_at = Utc::now() + Duration::minutes(minutes);

    let claims = ShareClaims {
        shared_by: user.username,
        file_id,
        exp: expires_at.timestamp() as usize,
    };

    let token = match encode(
        &JwtHeader::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to create share link"));
        }
    };

    HttpResponse::Ok().json(ApiResponse::ok(
        "Share link created",
        ShareResponse { token, expires_at },
    ))
}

async fn list_user_files(user: Option<AuthUser>, pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    if user.is_none() {
        return HttpResponse::Forbidden().json(ApiResponse::error("User not logged in"));
    }

    let user = user.unwrap();

    let files_res = sqlx::query_as::<_, FileRecord>(
        "SELECT id, file_name, mime_type, file_size, checksum, owner_uname FROM files WHERE owner_uname = ? AND deleted_at IS NULL LIMIT 50",
    )
    .bind(user.username)
    .fetch_all(pool.as_ref())
    .await;

    if let Err(e) = files_res {
        return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
    }

    HttpResponse::Ok().json(ApiResponse::ok(
        "files fetched successfully",
        files_res.unwrap(),
    ))
}

async fn search_user_files(
    user: Option<AuthUser>,
    pool: web::Data<Pool<Sqlite>>,
    query: web::Query<FileSearchQuery>,
) -> impl Responder {
    if user.is_none() {
        return HttpResponse::Forbidden().json(ApiResponse::error("User not logged in"));
    }

    let user = user.unwrap();

    let query = query.into_inner();
    let pattern = format!("%{}%", query.filename);

    let files_res = sqlx::query_as::<_, FileRecord>(
        "SELECT id, file_name, mime_type, file_size, checksum, owner_uname FROM files WHERE owner_uname = ? AND deleted_at IS NULL AND (file_name LIKE '%?%' OR file_path LIKE '%?%') LIMIT 50",
    )
    .bind(user.username)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool.as_ref())
    .await;

    if let Err(e) = files_res {
        return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
    }

    HttpResponse::Ok().json(ApiResponse::ok(
        "files fetched successfully",
        files_res.unwrap(),
    ))
}

pub fn register(config: &mut ServiceConfig, app_config: &AppConfig) {
    config.service(
        web::scope("/upload")
            .app_data(web::PayloadConfig::new(app_config.tus_max_size as usize))
            .wrap(from_fn(tus_resumable))
            .route(
                "",
                web::route().guard(guard::Options()).to(get_server_config),
            )
            .route("", web::post().to(create_upload))
            .route("/{upload_id}", web::head().to(get_upload_offset))
            .route("/{upload_id}", web::patch().to(upload_chunk)),
    );

    config
        .route("/{id}/download", web::get().to(download_file))
        .route("/{id}/share", web::post().to(create_share_link))
        .route("/my", web::get().to(list_user_files))
        .route("/search", web::get().to(search_user_files));
}
