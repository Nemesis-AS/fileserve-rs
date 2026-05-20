use actix_web::{FromRequest, HttpRequest, HttpResponse, ResponseError, dev::Payload};
use std::future::{Ready, ready};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::routes::api::types::ApiResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub username: String,
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
        let Some(cookie) = req.cookie("auth_token") else {
            return ready(Err(AuthError("Missing auth token".into()).into()));
        };

        // @todo! Get secret from config/env
        match decode::<JwtClaims>(
            cookie.value(),
            &DecodingKey::from_secret("secret".as_ref()),
            &Validation::default(),
        ) {
            Ok(data) => ready(Ok(AuthUser { username: data.claims.username })),
            Err(_) => ready(Err(AuthError("Invalid or expired token".into()).into())),
        }
    }
}
