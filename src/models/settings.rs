use std::env::var;
use std::sync::RwLock;

use serde::Serialize;
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// Binary GB (GiB) — matches the frontend's 1024-based sizing.
const BYTES_PER_GB: f64 = 1_073_741_824.0;

/// Seed defaults used only when the settings row doesn't yet exist. Env vars
/// still seed a fresh database (so an existing `.env` deployment migrates its
/// values in on first boot), but after that the database is the source of
/// truth and these env vars are ignored.
const DEFAULT_STORAGE_PATH: &str = "./files";
const DEFAULT_TUS_MAX_SIZE: i64 = 5_368_709_120; // 5 GiB
const DEFAULT_QUOTA_GB: f64 = 20.0;

/// The admin-editable settings, as stored in the DB and returned to the UI.
/// The JWT secret lives in the same DB row but is deliberately *not* part of
/// this struct, so it can never leak through the settings API.
#[derive(Clone, Serialize, FromRow)]
pub struct SettingsData {
    pub storage_path: String,
    pub tus_max_size: i64,
    pub default_quota_bytes: i64,
}

/// A partial update from the admin API. Every field is optional so callers can
/// patch one setting without restating the others.
#[derive(serde::Deserialize)]
pub struct SettingsPatch {
    pub storage_path: Option<String>,
    pub tus_max_size: Option<i64>,
    pub default_quota_bytes: Option<i64>,
}

/// Full settings row as persisted, including the secret. Used only inside this
/// module to bootstrap and load; never handed out.
#[derive(FromRow)]
struct SettingsRow {
    jwt_secret: String,
    storage_path: String,
    tus_max_size: i64,
    default_quota_bytes: i64,
}

/// Live, shared, mutable settings. Stored in `web::Data` so every worker sees
/// one instance; reads take a brief lock and clone out, writes are rare (admin
/// edits only), so a plain `RwLock` is more than sufficient.
pub struct Settings {
    inner: RwLock<SettingsData>,
}

impl Settings {
    /// Loads settings from the DB, creating the row (and generating a JWT
    /// secret) on first run. Returns the live handle plus the JWT secret, which
    /// is immutable at runtime and so lives in [`AppConfig`] rather than here.
    pub async fn load_or_init(pool: &Pool<Sqlite>) -> Result<(Self, String), sqlx::Error> {
        let existing: Option<SettingsRow> =
            sqlx::query_as("SELECT jwt_secret, storage_path, tus_max_size, default_quota_bytes \
                 FROM settings WHERE id = 1")
                .fetch_optional(pool)
                .await?;

        if let Some(row) = existing {
            let data = SettingsData {
                storage_path: row.storage_path,
                tus_max_size: row.tus_max_size,
                default_quota_bytes: row.default_quota_bytes,
            };
            return Ok((Self { inner: RwLock::new(data) }, row.jwt_secret));
        }

        // Fresh database: seed from env-or-defaults and mint a secret.
        let data = SettingsData {
            storage_path: var("STORAGE_PATH")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_STORAGE_PATH.to_string()),
            tus_max_size: var("TUS_MAX_SIZE")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(DEFAULT_TUS_MAX_SIZE),
            default_quota_bytes: quota_gb_to_bytes(
                var("DEFAULT_QUOTA_GB")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .filter(|gb| *gb >= 0.0)
                    .unwrap_or(DEFAULT_QUOTA_GB),
            ),
        };
        let jwt_secret = generate_secret();

        sqlx::query(
            "INSERT INTO settings (id, jwt_secret, storage_path, tus_max_size, default_quota_bytes) \
             VALUES (1, ?, ?, ?, ?)",
        )
        .bind(&jwt_secret)
        .bind(&data.storage_path)
        .bind(data.tus_max_size)
        .bind(data.default_quota_bytes)
        .execute(pool)
        .await?;

        Ok((Self { inner: RwLock::new(data) }, jwt_secret))
    }

    pub fn snapshot(&self) -> SettingsData {
        self.inner.read().expect("settings lock poisoned").clone()
    }

    pub fn storage_path(&self) -> String {
        self.inner.read().expect("settings lock poisoned").storage_path.clone()
    }

    pub fn tus_max_size(&self) -> i64 {
        self.inner.read().expect("settings lock poisoned").tus_max_size
    }

    pub fn default_quota_bytes(&self) -> i64 {
        self.inner.read().expect("settings lock poisoned").default_quota_bytes
    }

    /// Applies a validated patch: persists to the DB first (the source of
    /// truth), then swaps the in-memory copy so live requests pick it up
    /// without a restart. No lock is held across the await.
    pub async fn update(
        &self,
        pool: &Pool<Sqlite>,
        patch: SettingsPatch,
    ) -> Result<SettingsData, sqlx::Error> {
        let mut data = self.snapshot();
        if let Some(v) = patch.storage_path {
            data.storage_path = v;
        }
        if let Some(v) = patch.tus_max_size {
            data.tus_max_size = v;
        }
        if let Some(v) = patch.default_quota_bytes {
            data.default_quota_bytes = v;
        }

        sqlx::query(
            "UPDATE settings SET storage_path = ?, tus_max_size = ?, default_quota_bytes = ? \
             WHERE id = 1",
        )
        .bind(&data.storage_path)
        .bind(data.tus_max_size)
        .bind(data.default_quota_bytes)
        .execute(pool)
        .await?;

        *self.inner.write().expect("settings lock poisoned") = data.clone();
        Ok(data)
    }
}

/// Converts a GB quota into bytes, rounding to the nearest byte.
fn quota_gb_to_bytes(gb: f64) -> i64 {
    (gb * BYTES_PER_GB).round() as i64
}

/// 256 bits of CSPRNG-backed randomness (two v4 UUIDs, hex, no dashes) — ample
/// for an HMAC signing key, and avoids pulling in a dedicated RNG crate since
/// `uuid`'s v4 already draws from the OS CSPRNG.
fn generate_secret() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
