use actix_web::HttpRequest;
use actix_web::cookie::{SameSite, time::Duration as CookieDuration};
use actix_web::{HttpResponse, Responder, cookie::Cookie, web};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

use crate::config::AppConfig;
use crate::demo;
use crate::extractors::JwtClaims;
use crate::routes::api::types::ApiResponse;
use crate::utils::throttle::{Throttles, throttle_key};

const SESSION_DAYS: i64 = 7;

/// Signs a session token for `username`, expiring at `expires_at`. Shared by
/// every route that starts a session so the claim shape stays in one place.
pub(crate) fn encode_session_token(
    username: &str,
    expires_at: chrono::DateTime<Utc>,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = JwtClaims {
        username: username.to_string(),
        exp: expires_at.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Builds the session cookie carrying `token`. Shared for the same reason as
/// [`encode_session_token`]: the attributes here are security-relevant and must
/// not drift between the routes that issue sessions.
pub(crate) fn session_cookie(
    token: String,
    max_age: CookieDuration,
    secure: bool,
) -> Cookie<'static> {
    Cookie::build("auth_token", token)
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(max_age)
        .finish()
}

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
    status: String,
    demo_expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize)]
struct UserResponse {
    id: String,
    username: String,
    name: String,
    role: String,
    avatar: Option<String>,
    /// Present so the client can show the demo banner and countdown straight
    /// after signing in, without a second round-trip. UI only: every demo
    /// restriction is enforced server-side and independently of this flag.
    demo: bool,
    #[serde(rename = "demoExpiresAt", skip_serializing_if = "Option::is_none")]
    demo_expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

async fn login(
    req: HttpRequest,
    body: web::Json<LoginRequestBody>,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    throttle: web::Data<Throttles>,
) -> impl Responder {
    // Bounded before touching the database, because verifying a password costs
    // a deliberate ~250ms of bcrypt. Unbounded, that is both a credential
    // guessing oracle and the cheapest way to saturate the server.
    let key = throttle_key(&req, config.trusted_proxy_hops);
    if let Err(retry_after) = throttle.login.check(&key) {
        return HttpResponse::TooManyRequests()
            .append_header(("Retry-After", retry_after.as_secs().max(1).to_string()))
            .json(ApiResponse::error(
                "Too many sign-in attempts. Try again shortly.",
            ));
    }

    let result = sqlx::query_as::<_, UserRow>(
        "SELECT username, name, password, role, avatar, status, demo_expires_at \
         FROM users WHERE username = ?",
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

    // A stored value that isn't a valid bcrypt hash is not a server fault: it
    // means this account cannot be signed into with a password at all, which is
    // exactly how demo accounts are stored. Answering 500 here would also tell
    // an attacker which accounts have unusable hashes, so both cases collapse
    // into the same 401 below.
    let password_match = verify(&body.password, &row.password).unwrap_or(false);

    if !password_match {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::error("Incorrect username or password"));
    }

    // Checked only after the password verifies, so we don't reveal which
    // accounts are suspended to an unauthenticated caller.
    if row.status == "suspended" {
        return HttpResponse::Forbidden()
            .json(ApiResponse::error("Your account has been suspended"));
    }

    let expires_at = Utc::now() + Duration::days(SESSION_DAYS);

    let token: String =
        match encode_session_token(&row.username, expires_at, &config.jwt_secret) {
            Ok(tk) => tk,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::error("An error occurred while authenticating"));
            }
        };

    let cookie = session_cookie(
        token.clone(),
        CookieDuration::days(SESSION_DAYS),
        config.cookie_secure,
    );

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
                demo: row.demo_expires_at.is_some(),
                demo_expires_at: row.demo_expires_at,
            },
        },
    ))
}

/// `POST /auth/demo` — mints a throwaway account and signs the caller in.
///
/// Only registered when demo mode is on, so on a normal deployment this route
/// does not exist at all rather than existing behind a runtime guard.
async fn start_demo(
    req: HttpRequest,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
    throttle: web::Data<Throttles>,
) -> impl Responder {
    let Some(demo) = config.demo.as_ref() else {
        return HttpResponse::NotFound().json(ApiResponse::error("Not found"));
    };

    let key = throttle_key(&req, config.trusted_proxy_hops);
    if let Err(retry_after) = throttle.provision.check(&key) {
        return HttpResponse::TooManyRequests()
            .append_header(("Retry-After", retry_after.as_secs().max(1).to_string()))
            .json(ApiResponse::error(
                "Too many demo accounts from your network. Try again in a few minutes.",
            ));
    }

    let (username, expires_at) = match demo::provision(pool.get_ref(), demo).await {
        Ok(created) => created,
        Err(demo::ProvisionError::AtCapacity) => {
            return HttpResponse::ServiceUnavailable()
                .append_header(("Retry-After", "300"))
                .json(ApiResponse::error(
                    "The demo is at capacity right now. Try again in a few minutes.",
                ));
        }
        Err(demo::ProvisionError::Database(e)) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    // The session is pinned to the account's own lifetime, so the cookie dies
    // at the same instant the data does. This, not the reaper, is what ends a
    // demo session; the reaper is only a janitor and runs on a coarse tick.
    let token = match encode_session_token(&username, expires_at, &config.jwt_secret) {
        Ok(token) => token,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to start the demo"));
        }
    };

    let max_age = CookieDuration::seconds(demo::session_duration(demo).as_secs() as i64);
    let cookie = session_cookie(token.clone(), max_age, config.cookie_secure);

    HttpResponse::Created().cookie(cookie).json(ApiResponse::ok(
        "Demo session started",
        AuthResponse {
            token,
            user: UserResponse {
                id: username.clone(),
                name: "Demo visitor".to_string(),
                username,
                role: "user".to_string(),
                avatar: None,
                demo: true,
                demo_expires_at: Some(expires_at),
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

pub fn register(config: &mut actix_web::web::ServiceConfig, app_config: &AppConfig) {
    config.route("/login", web::post().to(login));
    config.route("/logout", web::post().to(logout));

    // Registered only for a demo deployment, so a self-hosted install has no
    // account-creation surface at all rather than one behind a runtime check.
    //
    // The `Throttle` itself is built in `main` and shared through `app_data`.
    // Building it here would put one behind every worker, and since requests
    // round-robin across workers each would only ever see a fraction of the
    // traffic, letting the real rate exceed the limit by a factor of N.
    if app_config.demo.is_some() {
        config.route("/demo", web::post().to(start_demo));
    }
}
