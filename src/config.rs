use std::env::var;

#[derive(Clone)]
pub struct AppConfig {
    pub storage_path: String,
    pub jwt_secret: String,
    pub tus_max_size: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let storage_path = var("STORAGE_PATH").unwrap_or(String::from("/files"));
        let jwt_secret = var("JWT_SECRET").unwrap_or(String::from("secret"));
        let tus_max_size = var("TUS_MAX_SIZE")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(5368709120 as u64);

        Self {
            storage_path,
            jwt_secret,
            tus_max_size,
        }
    }
}
