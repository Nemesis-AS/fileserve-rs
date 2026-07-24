#[cfg(debug_assertions)]
const DEFAULT_ALLOWED_ORIGINS: &[&str] = &["http://localhost:5173", "http://127.0.0.1:5173"];

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    #[cfg(debug_assertions)]
    pub allowed_origins: Vec<String>,
    pub max_payload_bytes: usize,
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
        }
    }
}
