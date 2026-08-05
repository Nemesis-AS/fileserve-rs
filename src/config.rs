use std::time::Duration;

use serde::Serialize;

#[cfg(debug_assertions)]
const DEFAULT_ALLOWED_ORIGINS: &[&str] = &["http://localhost:5173", "http://127.0.0.1:5173"];

pub const DEFAULT_UPDATE_REPO: &str = "Nemesis-AS/fileserve-rs";
pub const DEFAULT_UPDATE_API_BASE: &str = "https://api.github.com";

/// Hard ceiling on a single buffered request body.
///
/// `upload_chunk` takes `web::Bytes`, so a TUS PATCH is held whole in memory.
/// Deriving the limit from `tus_max_size` alone (5 GiB by default) means one
/// crafted request can exhaust RAM. The client uploads in 5 MiB chunks, so this
/// clears real traffic by a wide margin.
const MAX_PAYLOAD_CEILING: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestartMode {
    /// Detect the environment and pick `Exit` under a supervisor, else `Spawn`.
    Auto,
    /// Launch a successor process, supervise its startup, then exit.
    Spawn,
    /// Exit and rely on an external supervisor to start the new binary.
    Exit,
}

impl RestartMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "spawn" => Some(Self::Spawn),
            "exit" => Some(Self::Exit),
            _ => None,
        }
    }

    pub fn effective(self) -> Self {
        match self {
            Self::Auto => {
                let supervised = std::env::var_os("INVOCATION_ID").is_some()
                    || std::path::Path::new("/.dockerenv").exists();
                if supervised { Self::Exit } else { Self::Spawn }
            }
            explicit => explicit,
        }
    }
}

/// Caps applied to a public demo deployment.
///
/// These live here, sourced from the environment at boot, rather than in the
/// admin-editable `settings` row on purpose: `PATCH /settings` must not be able
/// to raise a demo limit. Every effective limit is the minimum of the two.
#[derive(Clone, Copy)]
pub struct DemoConfig {
    /// How long a provisioned account and its files survive.
    pub user_ttl: Duration,
    pub quota_bytes: i64,
    pub max_upload_bytes: i64,
    /// Live demo accounts before provisioning starts refusing.
    pub max_accounts: i64,
    /// Total bytes held by all demo accounts before provisioning refuses.
    /// Refusing a new visitor is a better failure than filling the disk.
    pub total_storage_bytes: i64,
    pub provision_per_ip_per_hour: u32,
    pub share_max_minutes: i64,
}

impl DemoConfig {
    fn load() -> Self {
        Self {
            user_ttl: Duration::from_secs(env_num("DEMO_USER_TTL_MINUTES", 120) as u64 * 60),
            quota_bytes: env_num("DEMO_QUOTA_MB", 200) * 1024 * 1024,
            max_upload_bytes: env_num("DEMO_MAX_UPLOAD_MB", 25) * 1024 * 1024,
            max_accounts: env_num("DEMO_MAX_ACCOUNTS", 50),
            total_storage_bytes: env_num("DEMO_TOTAL_STORAGE_GB", 10) * 1024 * 1024 * 1024,
            provision_per_ip_per_hour: env_num("DEMO_PROVISION_PER_IP_PER_HOUR", 3) as u32,
            share_max_minutes: env_num("DEMO_SHARE_MAX_MINUTES", 60),
        }
    }
}

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    #[cfg(debug_assertions)]
    pub allowed_origins: Vec<String>,
    pub max_payload_bytes: usize,
    pub update_repo: String,
    pub update_api_base: String,
    pub self_update_enabled: bool,
    pub restart_mode: RestartMode,
    /// Whether session cookies carry `Secure`, and whether HSTS is sent.
    ///
    /// Off unless asked for. A `Secure` cookie is dropped outright over plain
    /// HTTP, and this server is routinely run exactly that way: `INSTALL.md`
    /// has you download a binary that listens on `0.0.0.0:8112` with no TLS in
    /// front. Defaulting this on would lock those users out of their own
    /// server, and pin a year of HSTS on a host with no HTTPS to fall back to.
    /// Deployments behind TLS set `COOKIE_SECURE=true`.
    pub cookie_secure: bool,
    /// `None` unless `DEMO_MODE` is on. Modelled as an `Option` rather than a
    /// flag beside loose fields so a demo cap simply cannot be read when demo
    /// mode is off, and every demo behaviour reads as `if let Some(demo)`.
    pub demo: Option<DemoConfig>,
    /// How many reverse proxies sit in front of us. Zero means `X-Forwarded-For`
    /// is ignored entirely, which is the only safe default for a server that
    /// might be directly exposed.
    pub trusted_proxy_hops: usize,
    /// Gap between maintenance sweeps.
    pub maintenance_interval: Duration,
}

impl AppConfig {
    /// The largest upload this deployment will accept, folding the demo cap
    /// into whatever the (admin-editable) settings row says.
    pub fn effective_max_upload(&self, settings_max: i64) -> i64 {
        match &self.demo {
            Some(demo) => settings_max.min(demo.max_upload_bytes),
            None => settings_max,
        }
    }
}

impl AppConfig {
    pub fn load(jwt_secret: String, max_payload_bytes: usize) -> Self {
        #[cfg(debug_assertions)]
        let allowed_origins = match std::env::var("ALLOWED_ORIGINS") {
            Ok(raw) => raw
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            Err(_) => DEFAULT_ALLOWED_ORIGINS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };

        Self {
            jwt_secret,
            #[cfg(debug_assertions)]
            allowed_origins,
            max_payload_bytes: max_payload_bytes.min(MAX_PAYLOAD_CEILING),
            update_repo: non_empty("UPDATE_REPO").unwrap_or_else(|| DEFAULT_UPDATE_REPO.to_string()),
            update_api_base: non_empty("UPDATE_API_BASE")
                .map(|base| base.trim_end_matches('/').to_string())
                .unwrap_or_else(|| DEFAULT_UPDATE_API_BASE.to_string()),
            self_update_enabled: env_flag("SELF_UPDATE_ENABLED", true),
            restart_mode: non_empty("UPDATE_RESTART_MODE")
                .and_then(|raw| RestartMode::parse(&raw))
                .unwrap_or(RestartMode::Auto),
            cookie_secure: env_flag("COOKIE_SECURE", false),
            demo: env_flag("DEMO_MODE", false).then(DemoConfig::load),
            trusted_proxy_hops: env_num("TRUSTED_PROXY_HOPS", 0).max(0) as usize,
            maintenance_interval: Duration::from_secs(
                env_num("MAINTENANCE_INTERVAL_SECS", 300).max(10) as u64,
            ),
        }
    }
}

fn non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// Reads a boolean env var, treating the usual falsey spellings as `false` and
/// anything else present as `true`.
pub(crate) fn env_flag(key: &str, default: bool) -> bool {
    non_empty(key)
        .map(|raw| !matches!(raw.trim(), "0" | "false" | "no" | "off"))
        .unwrap_or(default)
}

/// Reads an integer env var, falling back on anything unparseable.
fn env_num(key: &str, default: i64) -> i64 {
    non_empty(key)
        .and_then(|raw| raw.trim().parse::<i64>().ok())
        .unwrap_or(default)
}
