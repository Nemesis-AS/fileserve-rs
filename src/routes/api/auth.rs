use actix_web::HttpRequest;
use actix_web::{HttpResponse, Responder, cookie::Cookie, web};
use bcrypt::verify;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

use crate::extractors::{AuthUser, JwtClaims};
use crate::routes::api::types::ApiResponse;

#[derive(Deserialize)]
struct LoginRequestBody {
    username: String,
    password: String,
}

#[derive(FromRow)]
struct UserCreds {
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
}

async fn login(body: web::Json<LoginRequestBody>, pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    let result = sqlx::query_as::<_, UserCreds>("SELECT password FROM users WHERE username = ?")
        .bind(&body.username)
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(row) => {
            let password_match = match verify(&body.password, &row.password) {
                Ok(verification_result) => verification_result,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::error("An error occurred while authenticating"));
                }
            };

            if password_match {
                let claims = JwtClaims {
                    username: body.username.clone(),
                };

                let token: String = match encode(
                    &Header::default(),
                    &claims,
                    // @todo! Get secret from config/env
                    &EncodingKey::from_secret("secret".as_ref()),
                ) {
                    Ok(tk) => tk,
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .json(ApiResponse::error("An error occured while authenticating"));
                    }
                };

                let cookie: Cookie = Cookie::build("auth_token", token).finish();

                HttpResponse::Ok().cookie(cookie).json(ApiResponse::ok(
                    "Authentication successful",
                    AuthResponse {
                        token: String::from("124"),
                    },
                ))
            } else {
                HttpResponse::Unauthorized().json(ApiResponse::error("Incorrect password"))
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(ApiResponse::error("User not found!"))
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(ApiResponse::error("An error occured while authenticating")),
    }
}

async fn logout(req: HttpRequest, _user: AuthUser) -> impl Responder {
    if let Some(mut auth_cookie) = req.cookie("auth_token") {
        auth_cookie.make_removal();
        HttpResponse::Ok()
            .cookie(auth_cookie)
            .json(ApiResponse::ok_msg("Logout successful"))
    } else {
        HttpResponse::BadRequest().json(ApiResponse::error("Failed to log user out"))
    }
}

pub fn register(config: &mut actix_web::web::ServiceConfig) {
    config.route("/login", web::post().to(login));
    config.route("/logout", web::post().to(logout));
}
