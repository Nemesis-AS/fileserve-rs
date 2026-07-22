use std::env::var;

/// Binary GB (GiB) — matches the frontend's 1024-based sizing.
const BYTES_PER_GB: f64 = 1_073_741_824.0;

/// Default storage quota, in GB, when `DEFAULT_QUOTA_GB` is unset.
const DEFAULT_QUOTA_GB: f64 = 20.0;

/// Cross-origin origins allowed by default when `ALLOWED_ORIGINS` is unset.
/// In production the frontend is embedded and served same-origin, so CORS is
/// only needed for the Vite dev server; these cover its default hosts.
const DEFAULT_ALLOWED_ORIGINS: &[&str] = &["http://localhost:5173", "http://127.0.0.1:5173"];

#[derive(Clone)]
pub struct AppConfig {
    pub storage_path: String,
    pub jwt_secret: String,
    pub tus_max_size: u64,
    /// Quota (in bytes) assigned to a new user when none is specified.
    pub default_quota_bytes: i64,
    /// Origins permitted to make credentialed cross-origin requests. Empty in a
    /// pure same-origin deployment; populated for dev (or a separately-hosted
    /// client) via `ALLOWED_ORIGINS`.
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let storage_path = var("STORAGE_PATH").unwrap_or(String::from("/files"));
        let jwt_secret = var("JWT_SECRET").unwrap_or(String::from("secret"));
        let tus_max_size = var("TUS_MAX_SIZE")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(5368709120 as u64);

        let default_quota_gb = var("DEFAULT_QUOTA_GB")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|gb| *gb >= 0.0)
            .unwrap_or(DEFAULT_QUOTA_GB);
        let default_quota_bytes = (default_quota_gb * BYTES_PER_GB).round() as i64;

        // Comma-separated allowlist; falls back to the dev-server origins. An
        // explicitly empty value (`ALLOWED_ORIGINS=`) means "no cross-origin
        // access", which is correct for a same-origin production deploy.
        let allowed_origins = match var("ALLOWED_ORIGINS") {
            Ok(raw) => raw
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            Err(_) => DEFAULT_ALLOWED_ORIGINS.iter().map(|s| s.to_string()).collect(),
        };

        Self {
            storage_path,
            jwt_secret,
            tus_max_size,
            default_quota_bytes,
            allowed_origins,
        }
    }
}
