use std::env::var;

use bcrypt::{DEFAULT_COST, hash};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

/// Default admin username when `ADMIN_USERNAME` isn't set.
const DEFAULT_ADMIN_USERNAME: &str = "admin";

/// The credentials chosen when an admin is seeded, surfaced to the caller so
/// `main` can print them on first boot — the generated password is stored only
/// as a hash, so this is the operator's one chance to see it.
pub struct SeededAdmin {
    pub username: String,
    /// `Some` only when the server generated the password; `None` when it came
    /// from `ADMIN_PASSWORD` (the operator already knows it).
    pub generated_password: Option<String>,
}

/// Ensures at least one admin exists, creating one on a fresh database so a bare
/// deploy is usable with zero manual setup. A no-op once any admin is present,
/// so it can run unconditionally on every boot. Username and password seed from
/// `ADMIN_USERNAME` / `ADMIN_PASSWORD`; a missing password is randomly generated
/// and returned for the operator to record.
pub async fn seed_admin(pool: &Pool<Sqlite>) -> Result<Option<SeededAdmin>, sqlx::Error> {
    let admin_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'admin'")
            .fetch_one(pool)
            .await?;
    if admin_count > 0 {
        return Ok(None);
    }

    let username = var("ADMIN_USERNAME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_ADMIN_USERNAME.to_string());

    let provided = var("ADMIN_PASSWORD")
        .ok()
        .filter(|p| !p.trim().is_empty());
    let generated = provided.is_none();
    let plain = provided.unwrap_or_else(|| Uuid::new_v4().simple().to_string());

    let hashed = hash(&plain, DEFAULT_COST)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to hash admin password: {e}")))?;

    // Quota stays NULL (unlimited) for the seeded admin, matching the schema's
    // intent. A UNIQUE conflict here means the name is taken by a non-admin, so
    // fall through silently rather than clobbering that account.
    let inserted = sqlx::query(
        "INSERT OR IGNORE INTO users(username, name, password, role, status) \
         VALUES(?, ?, ?, 'admin', 'active')",
    )
    .bind(&username)
    .bind(&username)
    .bind(&hashed)
    .execute(pool)
    .await?;

    if inserted.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(SeededAdmin {
        username,
        generated_password: generated.then_some(plain),
    }))
}
