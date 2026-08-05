//! Ephemeral accounts for a public demo deployment.
//!
//! Everything here is inert unless `DEMO_MODE` is set. The provisioning route
//! is not even registered otherwise, so a self-hosted install has no extra
//! surface at all.

use std::fs;
use std::time::Duration;

use sqlx::{Pool, Sqlite, SqlitePool};
use uuid::Uuid;

use crate::config::{AppConfig, DemoConfig};
use crate::models::Settings;
use crate::utils::storage::release_blob_if_unreferenced;
use crate::utils::tus::upload_file_path;

/// Stored in `users.password` for demo accounts.
///
/// Deliberately *not* a bcrypt hash. These accounts never sign in with a
/// password (the provisioning route hands back a session directly), and hashing
/// at the usual cost is roughly 250ms of CPU per call, which would turn an
/// unauthenticated endpoint into a cheap way to saturate the server from many
/// addresses at once, defeating the per-IP throttle by design.
///
/// `bcrypt::verify` returns `Err` against this, and `login` maps that to the
/// same 401 as a wrong password, so password login is structurally impossible
/// here rather than merely improbable.
pub const DEMO_PASSWORD_SENTINEL: &str = "!demo-account-no-password-login";

/// Rows the reaper retires per tick, so one pass can't hold the write lock
/// for an unbounded time. Anything left over is picked up on the next tick.
const REAP_BATCH: i64 = 50;

/// Outcome of asking for a demo account.
pub enum ProvisionError {
    /// Too many live demo accounts, or their files already fill the budget.
    AtCapacity,
    Database(sqlx::Error),
}

/// Whether the deployment can take another demo visitor right now.
///
/// Two independent ceilings. The account cap bounds concurrency; the storage
/// cap bounds disk, because `max_accounts * quota_bytes` is a worst case that
/// will not fit on a small box. Refusing a visitor is a far better failure than
/// filling the disk out from under the rest of the server.
async fn has_capacity(pool: &Pool<Sqlite>, demo: &DemoConfig) -> Result<bool, sqlx::Error> {
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users \
         WHERE demo_expires_at IS NOT NULL AND datetime(demo_expires_at) > datetime('now')",
    )
    .fetch_one(pool)
    .await?;

    if live >= demo.max_accounts {
        return Ok(false);
    }

    let used: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(f.file_size), 0) FROM files f \
         JOIN users u ON u.username = f.owner_uname \
         WHERE u.demo_expires_at IS NOT NULL AND f.deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    Ok(used < demo.total_storage_bytes)
}

/// Creates a demo account and returns its username and expiry.
pub async fn provision(
    pool: &Pool<Sqlite>,
    demo: &DemoConfig,
) -> Result<(String, chrono::DateTime<chrono::Utc>), ProvisionError> {
    if !has_capacity(pool, demo)
        .await
        .map_err(ProvisionError::Database)?
    {
        return Err(ProvisionError::AtCapacity);
    }

    let expires_at =
        chrono::Utc::now() + chrono::Duration::from_std(demo.user_ttl).unwrap_or_default();

    // The PK enforces uniqueness; a v4 collision on 10 hex chars is remote but
    // retrying is cheaper than reasoning about whether it can happen.
    for _ in 0..3 {
        let username = format!("demo_{}", &Uuid::new_v4().simple().to_string()[..10]);

        let insert = sqlx::query(
            "INSERT INTO users(username, name, password, role, status, quota_bytes, demo_expires_at) \
             VALUES(?, 'Demo visitor', ?, 'user', 'active', ?, ?)",
        )
        .bind(&username)
        .bind(DEMO_PASSWORD_SENTINEL)
        .bind(demo.quota_bytes)
        .bind(expires_at)
        .execute(pool)
        .await;

        match insert {
            Ok(_) => return Ok((username, expires_at)),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => continue,
            Err(e) => return Err(ProvisionError::Database(e)),
        }
    }

    Err(ProvisionError::Database(sqlx::Error::Protocol(
        "could not allocate a demo username".into(),
    )))
}

/// Starts the background janitor.
///
/// Must be called from `main`, **not** from inside the `HttpServer::new`
/// factory: that closure runs once per worker, which would leave several
/// reapers racing each other over the same rows.
///
/// This is `actix_web::rt::spawn` rather than the OS thread that
/// `update::spawn_checker` uses. That one is a blocking thread because `ureq`
/// is blocking; sqlx is async and its pool is bound to the runtime actix
/// drives, so it needs a task here.
pub fn spawn_maintenance(pool: SqlitePool, settings: actix_web::web::Data<Settings>, config: AppConfig) {
    let interval = config.maintenance_interval;

    actix_web::rt::spawn(async move {
        loop {
            // Work first, sleep second. A server that was down over the weekend
            // must not leave expired accounts standing until one interval after
            // it comes back.
            let storage_path = settings.storage_path();

            if let Err(e) = sweep_stale_uploads(&pool, &storage_path).await {
                eprintln!("WARN: stale-upload sweep failed: {e}");
            }

            if config.demo.is_some()
                && let Err(e) = reap_demo_accounts(&pool, &storage_path).await
            {
                eprintln!("WARN: demo reaper failed: {e}");
            }

            actix_web::rt::time::sleep(interval).await;
        }
    });
}

/// Deletes demo accounts past their expiry, along with everything they own.
///
/// Split from the tick loop with no timing inside it so it can be driven
/// directly from a test.
pub async fn reap_demo_accounts(pool: &Pool<Sqlite>, storage_path: &str) -> Result<(), sqlx::Error> {
    loop {
        let expired: Vec<String> = sqlx::query_scalar(
            "SELECT username FROM users \
             WHERE demo_expires_at IS NOT NULL AND datetime(demo_expires_at) <= datetime('now') \
             LIMIT ?",
        )
        .bind(REAP_BATCH)
        .fetch_all(pool)
        .await?;

        if expired.is_empty() {
            return Ok(());
        }

        let count = expired.len();
        for username in expired {
            reap_one(pool, storage_path, &username).await?;
        }

        if (count as i64) < REAP_BATCH {
            return Ok(());
        }
    }
}

async fn reap_one(
    pool: &Pool<Sqlite>,
    storage_path: &str,
    username: &str,
) -> Result<(), sqlx::Error> {
    // Rows go first and disk second, and the transaction commits before a
    // single file is touched. Crashing between the two leaves orphaned blobs,
    // which waste space and are collectable. The reverse order would leave
    // `files` rows pointing at bytes that are gone, and because blobs are
    // shared those rows can belong to somebody else.
    let mut tx = pool.begin().await?;

    let upload_ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM uploads WHERE owner_uname = ?")
            .bind(username)
            .fetch_all(&mut *tx)
            .await?;

    // Children before the parent: foreign keys are enforced (sqlx turns them on
    // for every connection), so deleting the user first fails outright.
    sqlx::query("DELETE FROM uploads WHERE owner_uname = ?")
        .bind(username)
        .execute(&mut *tx)
        .await?;

    let checksums: Vec<String> =
        sqlx::query_scalar("DELETE FROM files WHERE owner_uname = ? RETURNING checksum")
            .bind(username)
            .fetch_all(&mut *tx)
            .await?;

    sqlx::query("DELETE FROM users WHERE username = ?")
        .bind(username)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // Deduplicate before asking about each blob: one owner can hold many rows
    // pointing at the same bytes.
    let mut seen: Vec<String> = checksums;
    seen.sort();
    seen.dedup();

    for checksum in seen {
        release_blob_if_unreferenced(pool, storage_path, &checksum).await?;
    }

    for id in upload_ids {
        remove_staged_upload(storage_path, &id).await;
    }

    Ok(())
}

/// Collects uploads that were started and abandoned.
///
/// Runs for every deployment, not just demo hosts. `create_upload` stamps a 60
/// minute expiry but nothing has ever collected them, and because it
/// pre-allocates the full file up front an abandoned upload holds real disk.
/// They also count against their owner's quota until collected.
pub async fn sweep_stale_uploads(
    pool: &Pool<Sqlite>,
    storage_path: &str,
) -> Result<(), sqlx::Error> {
    let stale: Vec<String> = sqlx::query_scalar(
        "DELETE FROM uploads \
         WHERE status != 'completed' \
           AND expires_at IS NOT NULL \
           AND datetime(expires_at) < datetime('now') \
         RETURNING id",
    )
    .fetch_all(pool)
    .await?;

    for id in stale {
        remove_staged_upload(storage_path, &id).await;
    }

    // Completed rows have already handed their bytes to `files`, so they are
    // pure bookkeeping and grow without bound. Keep a week for debugging.
    sqlx::query(
        "DELETE FROM uploads \
         WHERE status = 'completed' \
           AND finished_at IS NOT NULL \
           AND datetime(finished_at) < datetime('now', '-7 days')",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn remove_staged_upload(storage_path: &str, upload_id: &str) {
    let path = upload_file_path(storage_path, upload_id);
    match actix_web::web::block(move || fs::remove_file(&path)).await {
        Ok(Err(e)) if e.kind() != std::io::ErrorKind::NotFound => {
            eprintln!("WARN: failed to remove staged upload {upload_id}: {e}");
        }
        Err(e) => eprintln!("WARN: staged-upload removal task failed for {upload_id}: {e}"),
        _ => {}
    }
}

/// Minutes a share link may last for `username`, given the demo ceiling and,
/// for a demo account, however long the account itself has left. A link that
/// outlives the file it points at is just a 404 with extra steps.
pub async fn share_ceiling_minutes(
    pool: &Pool<Sqlite>,
    demo: &DemoConfig,
    username: &str,
) -> Result<i64, sqlx::Error> {
    let expires: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT demo_expires_at FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await?
            .flatten();

    let mut ceiling = demo.share_max_minutes;
    if let Some(expires) = expires {
        let left = (expires - chrono::Utc::now()).num_minutes();
        ceiling = ceiling.min(left.max(1));
    }

    Ok(ceiling)
}

/// How long a provisioned session lasts, as a cookie duration.
pub fn session_duration(demo: &DemoConfig) -> Duration {
    demo.user_ttl
}
