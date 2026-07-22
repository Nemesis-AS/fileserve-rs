use actix_web::HttpRequest;
use actix_web::cookie::{SameSite, time::Duration as CookieDuration};
use actix_web::{HttpResponse, Responder, cookie::Cookie, web};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

use crate::config::AppConfig;
use crate::extractors::JwtClaims;
use crate::routes::api::types::ApiResponse;

const SESSION_DAYS: i64 = 7;

#[derive(Deserialize)]
struct LoginRequestBody {
    username: String,
    password: String,
}

#[derive(FromRow)]
struct UserRow {
    username: String,
    name: Option<String>,
    password: String,
    role: String,
    avatar: Option<String>,
}

#[derive(Serialize)]
struct UserResponse {
    id: String,
    username: String,
    name: String,
    role: String,
    avatar: Option<String>,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

async fn login(
    body: web::Json<LoginRequestBody>,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let result = sqlx::query_as::<_, UserRow>(
        "SELECT username, name, password, role, avatar FROM users WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_one(pool.get_ref())
    .await;

    let row = match result {
        Ok(row) => row,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::error("Incorrect username or password"));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("An error occurred while authenticating"));
        }
    };

    let password_match = match verify(&body.password, &row.password) {
        Ok(verification_result) => verification_result,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("An error occurred while authenticating"));
        }
    };

    if !password_match {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::error("Incorrect username or password"));
    }

    let claims = JwtClaims {
        username: row.username.clone(),
        exp: (Utc::now() + Duration::days(SESSION_DAYS)).timestamp() as usize,
    };

    let token: String = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    ) {
        Ok(tk) => tk,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("An error occurred while authenticating"));
        }
    };

    let cookie: Cookie = Cookie::build("auth_token", token.clone())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::days(SESSION_DAYS))
        .finish();

    HttpResponse::Ok().cookie(cookie).json(ApiResponse::ok(
        "Authentication successful",
        AuthResponse {
            token,
            user: UserResponse {
                id: row.username.clone(),
                name: row.name.unwrap_or_else(|| row.username.clone()),
                username: row.username,
                role: row.role,
                avatar: row.avatar,
            },
        },
    ))
}

/// Clears the session cookie. Deliberately does not require a valid token: a
/// client with an expired or malformed cookie still needs a way to sign out.
async fn logout(req: HttpRequest) -> impl Responder {
    let mut cookie = req
        .cookie("auth_token")
        .unwrap_or_else(|| Cookie::build("auth_token", "").path("/").finish());
    cookie.set_path("/");
    cookie.make_removal();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(ApiResponse::ok_msg("Logout successful"))
}

pub fn register(config: &mut actix_web::web::ServiceConfig) {
    config.route("/login", web::post().to(login));
    config.route("/logout", web::post().to(logout));
}
