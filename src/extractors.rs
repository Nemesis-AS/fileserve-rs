use actix_web::{FromRequest, HttpRequest, HttpResponse, ResponseError, dev::Payload, web};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};

use crate::{config::AppConfig, routes::api::types::ApiResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareClaims {
    pub shared_by: String,
    pub file_id: String,
    pub exp: usize,
}

pub fn verify_share_token(token: &str, secret: &str, expected_file_id: &str) -> bool {
    decode::<ShareClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims.file_id == expected_file_id)
    .unwrap_or(false)
}

pub struct AuthUser {
    pub username: String,
}

#[derive(Debug)]
struct AuthError(String);

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Unauthorized().json(ApiResponse::error(&self.0))
    }
}

impl FromRequest for AuthUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let Some(config) = req.app_data::<web::Data<AppConfig>>() else {
            return ready(Err(AuthError("Missing app config".into()).into()));
        };

        let Some(cookie) = req.cookie("auth_token") else {
            return ready(Err(AuthError("Missing auth token".into()).into()));
        };

        match decode::<JwtClaims>(
            cookie.value(),
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => ready(Ok(AuthUser {
                username: data.claims.username,
            })),
            Err(_) => ready(Err(AuthError("Invalid or expired token".into()).into())),
        }
    }
}
