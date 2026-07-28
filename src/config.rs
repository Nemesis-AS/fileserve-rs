use serde::Serialize;

#[cfg(debug_assertions)]
const DEFAULT_ALLOWED_ORIGINS: &[&str] = &["http://localhost:5173", "http://127.0.0.1:5173"];

pub const DEFAULT_UPDATE_REPO: &str = "Nemesis-AS/fileserve-rs";
pub const DEFAULT_UPDATE_API_BASE: &str = "https://api.github.com";

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
            max_payload_bytes,
            update_repo: non_empty("UPDATE_REPO").unwrap_or_else(|| DEFAULT_UPDATE_REPO.to_string()),
            update_api_base: non_empty("UPDATE_API_BASE")
                .map(|base| base.trim_end_matches('/').to_string())
                .unwrap_or_else(|| DEFAULT_UPDATE_API_BASE.to_string()),
            self_update_enabled: non_empty("SELF_UPDATE_ENABLED")
                .map(|raw| !matches!(raw.trim(), "0" | "false" | "no" | "off"))
                .unwrap_or(true),
            restart_mode: non_empty("UPDATE_RESTART_MODE")
                .and_then(|raw| RestartMode::parse(&raw))
                .unwrap_or(RestartMode::Auto),
        }
    }
}

fn non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}
