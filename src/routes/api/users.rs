use actix_web::{
    HttpResponse, Responder,
    web::{self, ServiceConfig},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::extractors::AuthUser;
use crate::routes::api::types::ApiResponse;

const MIN_PASSWORD_LEN: usize = 8;

/// Binary GB (GiB), matching the frontend's `fmtSize` (1024-based) so quota and
/// usage round-trip consistently between UI and DB.
const BYTES_PER_GB: f64 = 1_073_741_824.0;

/// Selects a user together with a live usage rollup. Storage used is computed
/// on the fly — `SUM(file_size)` over the owner's non-trashed files — rather
/// than stored, so it can never drift from the actual file rows. The password
/// hash is deliberately never selected so it can't leak into a response.
const USER_SELECT: &str = "SELECT u.username, u.name, u.role, u.avatar, u.status, u.quota_bytes, \
     COALESCE(SUM(f.file_size), 0) AS used_bytes, \
     COUNT(f.id) AS file_count \
     FROM users u \
     LEFT JOIN files f ON f.owner_uname = u.username AND f.deleted_at IS NULL";

#[derive(FromRow)]
struct UserRow {
    username: String,
    name: Option<String>,
    role: String,
    avatar: Option<String>,
    status: String,
    quota_bytes: Option<i64>,
    used_bytes: i64,
    file_count: i64,
}

/// Public shape of a user. `id` mirrors `username` to match the convention the
/// auth routes already use (the frontend keys users by `id`). Quota and usage
/// are exposed in GB to match the admin UI.
#[derive(Serialize)]
struct UserDto {
    id: String,
    username: String,
    name: String,
    role: String,
    avatar: Option<String>,
    status: String,
    #[serde(rename = "quotaGB")]
    quota_gb: Option<f64>,
    #[serde(rename = "usedGB")]
    used_gb: f64,
    files: i64,
}

impl From<UserRow> for UserDto {
    fn from(row: UserRow) -> Self {
        let name = row.name.unwrap_or_else(|| row.username.clone());
        Self {
            id: row.username.clone(),
            username: row.username,
            name,
            role: row.role,
            avatar: row.avatar,
            status: row.status,
            quota_gb: row.quota_bytes.map(|b| b as f64 / BYTES_PER_GB),
            used_gb: row.used_bytes as f64 / BYTES_PER_GB,
            files: row.file_count,
        }
    }
}

/// Fetches a single user with usage rollup — used to build the response after a
/// create/update, whose `RETURNING` clause can't carry the joined aggregate.
async fn fetch_user(pool: &Pool<Sqlite>, username: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(&format!(
        "{USER_SELECT} WHERE u.username = ? GROUP BY u.username"
    ))
    .bind(username)
    .fetch_optional(pool)
    .await
}

fn is_valid_role(role: &str) -> bool {
    role == "user" || role == "admin"
}

fn is_valid_status(status: &str) -> bool {
    status == "active" || status == "suspended"
}

/// Converts a GB quota from the API into stored bytes. `None` (no quota sent)
/// maps to `None` = unlimited.
fn quota_gb_to_bytes(gb: Option<f64>) -> Option<i64> {
    gb.map(|g| (g * BYTES_PER_GB).round() as i64)
}

/// Resolves the caller's role and rejects non-admins. Returns the offending
/// `HttpResponse` in the `Err` arm so handlers can early-return it directly.
async fn require_admin(pool: &Pool<Sqlite>, username: &str) -> Result<(), HttpResponse> {
    let role: Option<String> = match sqlx::query_scalar("SELECT role FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
    {
        Ok(role) => role,
        Err(e) => {
            return Err(HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())));
        }
    };

    match role.as_deref() {
        Some("admin") => Ok(()),
        // A missing user means the token outlived the account; treat as unauthorized.
        None => Err(HttpResponse::Unauthorized().json(ApiResponse::error("User no longer exists"))),
        Some(_) => {
            Err(HttpResponse::Forbidden().json(ApiResponse::error("Admin access required")))
        }
    }
}

/// `GET /auth/me` — the currently authenticated user, so a page refresh can
/// rehydrate auth from the cookie instead of trusting client storage.
async fn me(auth: AuthUser, pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    match fetch_user(pool.get_ref(), &auth.username).await {
        Ok(Some(row)) => {
            HttpResponse::Ok().json(ApiResponse::ok("Current user", UserDto::from(row)))
        }
        Ok(None) => {
            HttpResponse::Unauthorized().json(ApiResponse::error("User no longer exists"))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

#[derive(Serialize)]
struct QuotaDto {
    #[serde(rename = "quotaGB")]
    quota_gb: Option<f64>,
    #[serde(rename = "usedGB")]
    used_gb: f64,
    files: i64,
}

/// `GET /auth/quota` — the caller's current quota and live usage. Kept separate
/// from the session/login payload so it always reflects the current DB state:
/// if an admin changes the quota mid-session, the next fetch here picks it up.
async fn quota(auth: AuthUser, pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    match fetch_user(pool.get_ref(), &auth.username).await {
        Ok(Some(row)) => HttpResponse::Ok().json(ApiResponse::ok(
            "Quota",
            QuotaDto {
                quota_gb: row.quota_bytes.map(|b| b as f64 / BYTES_PER_GB),
                used_gb: row.used_bytes as f64 / BYTES_PER_GB,
                files: row.file_count,
            },
        )),
        Ok(None) => {
            HttpResponse::Unauthorized().json(ApiResponse::error("User no longer exists"))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

#[derive(Deserialize)]
struct UpdateMeBody {
    name: Option<String>,
}

/// `PATCH /auth/me` — update your own profile. Deliberately narrow: only the
/// display name is self-editable here. Role, status, quota and username stay
/// admin-only (via `PATCH /users/{id}`) so a user can't escalate or rename
/// their account. A blank/whitespace name is rejected rather than silently
/// clearing the stored name.
async fn update_me(
    auth: AuthUser,
    body: web::Json<UpdateMeBody>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    let name = body.name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if name.is_none() {
        return HttpResponse::BadRequest().json(ApiResponse::error("Name cannot be empty"));
    }

    let result = sqlx::query("UPDATE users SET name = COALESCE(?, name) WHERE username = ?")
        .bind(name)
        .bind(&auth.username)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => {
            HttpResponse::Unauthorized().json(ApiResponse::error("User no longer exists"))
        }
        Ok(_) => match fetch_user(pool.get_ref(), &auth.username).await {
            Ok(Some(row)) => {
                HttpResponse::Ok().json(ApiResponse::ok("Profile updated", UserDto::from(row)))
            }
            Ok(None) => {
                HttpResponse::Unauthorized().json(ApiResponse::error("User no longer exists"))
            }
            Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

#[derive(Deserialize)]
struct ChangePasswordBody {
    current_password: String,
    new_password: String,
}

/// `POST /auth/password` — change your own password. Requires the current
/// password so a stolen cookie alone can't lock the owner out.
async fn change_password(
    auth: AuthUser,
    body: web::Json<ChangePasswordBody>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    if body.new_password.len() < MIN_PASSWORD_LEN {
        return HttpResponse::BadRequest().json(ApiResponse::error(
            "New password must be at least 8 characters",
        ));
    }

    let current_hash: Option<String> =
        match sqlx::query_scalar("SELECT password FROM users WHERE username = ?")
            .bind(&auth.username)
            .fetch_optional(pool.get_ref())
            .await
        {
            Ok(hash) => hash,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::error(&e.to_string()));
            }
        };

    let Some(current_hash) = current_hash else {
        return HttpResponse::Unauthorized().json(ApiResponse::error("User no longer exists"));
    };

    match verify(&body.current_password, &current_hash) {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::error("Current password is incorrect"));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to verify password"));
        }
    }

    let new_hash = match hash(&body.new_password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to hash password"));
        }
    };

    match sqlx::query("UPDATE users SET password = ? WHERE username = ?")
        .bind(&new_hash)
        .bind(&auth.username)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok_msg("Password updated")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

/// `GET /users` — list every account.
async fn list_users(auth: AuthUser, pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    let rows = sqlx::query_as::<_, UserRow>(&format!(
        "{USER_SELECT} GROUP BY u.username ORDER BY u.username"
    ))
    .fetch_all(pool.get_ref())
    .await;

    match rows {
        Ok(rows) => {
            let users: Vec<UserDto> = rows.into_iter().map(UserDto::from).collect();
            HttpResponse::Ok().json(ApiResponse::ok("Users fetched", users))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

/// Create response: the new user, plus the temporary password when the server
/// generated one. `temp_password` is the only chance the admin gets to see it
/// (it's stored only as a hash), so it's surfaced as a distinct field rather
/// than buried in the message — the UI displays it for copying. Omitted
/// entirely when the admin supplied their own password.
#[derive(Serialize)]
struct CreatedUserDto {
    #[serde(flatten)]
    user: UserDto,
    #[serde(rename = "tempPassword", skip_serializing_if = "Option::is_none")]
    temp_password: Option<String>,
}

#[derive(Deserialize)]
struct CreateUserBody {
    username: String,
    name: Option<String>,
    password: Option<String>,
    role: Option<String>,
    avatar: Option<String>,
    status: Option<String>,
    #[serde(rename = "quotaGB")]
    quota_gb: Option<f64>,
}

/// `POST /users` — create an account.
async fn create_user(
    auth: AuthUser,
    body: web::Json<CreateUserBody>,
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    let username = body.username.trim();
    if username.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::error("Username is required"));
    }
    if username.chars().any(char::is_whitespace) {
        return HttpResponse::BadRequest()
            .json(ApiResponse::error("Username cannot contain spaces"));
    }

    let role = body.role.as_deref().unwrap_or("user");
    if !is_valid_role(role) {
        return HttpResponse::BadRequest().json(ApiResponse::error("Invalid role"));
    }

    let status = body.status.as_deref().unwrap_or("active");
    if !is_valid_status(status) {
        return HttpResponse::BadRequest().json(ApiResponse::error("Invalid status"));
    }

    // Fall back to the server's configured default when no quota is supplied.
    let quota_bytes = match quota_gb_to_bytes(body.quota_gb) {
        Some(b) if b < 0 => {
            return HttpResponse::BadRequest().json(ApiResponse::error("Quota cannot be negative"));
        }
        Some(b) => b,
        None => config.default_quota_bytes,
    };

    // Use the supplied password, or mint a temporary one when it's blank.
    let provided = body
        .password
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty());
    if let Some(p) = provided {
        if p.len() < MIN_PASSWORD_LEN {
            return HttpResponse::BadRequest().json(ApiResponse::error(
                "Password must be at least 8 characters",
            ));
        }
    }
    let generated = provided.is_none();
    let plain = provided
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string());

    let hashed = match hash(&plain, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("Failed to hash password"));
        }
    };

    let name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let avatar = body.avatar.as_deref();

    let insert = sqlx::query(
        "INSERT INTO users(username, name, password, role, avatar, status, quota_bytes) \
         VALUES(?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(username)
    .bind(name)
    .bind(&hashed)
    .bind(role)
    .bind(avatar)
    .bind(status)
    .bind(quota_bytes)
    .execute(pool.get_ref())
    .await;

    match insert {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return HttpResponse::Conflict()
                .json(ApiResponse::error("A user with that username already exists"));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    }

    let dto = match fetch_user(pool.get_ref(), username).await {
        Ok(Some(row)) => UserDto::from(row),
        Ok(None) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::error("User created but could not be loaded"));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };

    let response = CreatedUserDto {
        user: dto,
        temp_password: generated.then_some(plain),
    };
    HttpResponse::Created().json(ApiResponse::ok("User created", response))
}

#[derive(Deserialize)]
struct UpdateUserBody {
    name: Option<String>,
    role: Option<String>,
    password: Option<String>,
    avatar: Option<String>,
    status: Option<String>,
    #[serde(rename = "quotaGB")]
    quota_gb: Option<f64>,
}

/// `PATCH /users/{id}` — update an account. Every field is optional; omitted
/// fields are preserved via `COALESCE`.
async fn update_user(
    auth: AuthUser,
    path: web::Path<String>,
    body: web::Json<UpdateUserBody>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    let target = path.into_inner();

    if let Some(role) = body.role.as_deref() {
        if !is_valid_role(role) {
            return HttpResponse::BadRequest().json(ApiResponse::error("Invalid role"));
        }
        // Guard against an admin accidentally stripping their own admin rights,
        // which could leave the server with no way back in.
        if target == auth.username && role != "admin" {
            return HttpResponse::Forbidden()
                .json(ApiResponse::error("You cannot remove your own admin role"));
        }
    }

    if let Some(status) = body.status.as_deref() {
        if !is_valid_status(status) {
            return HttpResponse::BadRequest().json(ApiResponse::error("Invalid status"));
        }
        // A suspended account can't log in, so suspending yourself is a lockout.
        if target == auth.username && status == "suspended" {
            return HttpResponse::Forbidden()
                .json(ApiResponse::error("You cannot suspend your own account"));
        }
    }

    let quota_bytes = quota_gb_to_bytes(body.quota_gb);
    if quota_bytes.is_some_and(|b| b < 0) {
        return HttpResponse::BadRequest().json(ApiResponse::error("Quota cannot be negative"));
    }

    let new_hash = match body
        .password
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        Some(p) => {
            if p.len() < MIN_PASSWORD_LEN {
                return HttpResponse::BadRequest().json(ApiResponse::error(
                    "Password must be at least 8 characters",
                ));
            }
            match hash(p, DEFAULT_COST) {
                Ok(hash) => Some(hash),
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::error("Failed to hash password"));
                }
            }
        }
        None => None,
    };

    let name = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let role = body.role.as_deref();
    let avatar = body.avatar.as_deref();
    let status = body.status.as_deref();

    // Omitted fields are preserved via COALESCE. An existing row always reports
    // one affected row, so `rows_affected() == 0` cleanly means "no such user".
    let result = sqlx::query(
        "UPDATE users SET \
           name = COALESCE(?, name), \
           role = COALESCE(?, role), \
           avatar = COALESCE(?, avatar), \
           password = COALESCE(?, password), \
           status = COALESCE(?, status), \
           quota_bytes = COALESCE(?, quota_bytes) \
         WHERE username = ?",
    )
    .bind(name)
    .bind(role)
    .bind(avatar)
    .bind(new_hash.as_deref())
    .bind(status)
    .bind(quota_bytes)
    .bind(&target)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => {
            HttpResponse::NotFound().json(ApiResponse::error("User not found"))
        }
        Ok(_) => match fetch_user(pool.get_ref(), &target).await {
            Ok(Some(row)) => {
                HttpResponse::Ok().json(ApiResponse::ok("User updated", UserDto::from(row)))
            }
            Ok(None) => HttpResponse::NotFound().json(ApiResponse::error("User not found")),
            Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
        },
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string())),
    }
}

/// `DELETE /users/{id}` — remove an account.
async fn delete_user(
    auth: AuthUser,
    path: web::Path<String>,
    pool: web::Data<Pool<Sqlite>>,
) -> impl Responder {
    if let Err(resp) = require_admin(pool.get_ref(), &auth.username).await {
        return resp;
    }

    let target = path.into_inner();

    // Self-deletion is the one path that could drop the last admin, so block it.
    if target == auth.username {
        return HttpResponse::BadRequest()
            .json(ApiResponse::error("You cannot delete your own account"));
    }

    // `files.owner_uname` references `users`, so a user with files must not be
    // removed — doing so would orphan (or fail on) those rows.
    let file_count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE owner_uname = ?")
        .bind(&target)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(count) => count,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
        }
    };
    if file_count > 0 {
        return HttpResponse::Conflict()
            .json(ApiResponse::error("Cannot delete a user who still owns files"));
    }

    let deleted: Option<String> =
        match sqlx::query_scalar("DELETE FROM users WHERE username = ? RETURNING username")
            .bind(&target)
            .fetch_optional(pool.get_ref())
            .await
        {
            Ok(deleted) => deleted,
            Err(e) => {
                return HttpResponse::InternalServerError().json(ApiResponse::error(&e.to_string()));
            }
        };

    match deleted {
        Some(_) => HttpResponse::Ok().json(ApiResponse::ok_msg("User deleted")),
        None => HttpResponse::NotFound().json(ApiResponse::error("User not found")),
    }
}

/// Account routes for the caller's own user — mounted under the `auth` scope.
pub fn register_account(config: &mut ServiceConfig) {
    config
        .route("/me", web::get().to(me))
        .route("/me", web::patch().to(update_me))
        .route("/quota", web::get().to(quota))
        .route("/password", web::post().to(change_password));
}

/// Admin user-management routes — mounted under the `users` scope.
pub fn register(config: &mut ServiceConfig) {
    config
        .route("", web::get().to(list_users))
        .route("", web::post().to(create_user))
        .route("/{id}", web::patch().to(update_user))
        .route("/{id}", web::delete().to(delete_user));
}
