use base64::{Engine, engine::general_purpose::STANDARD};
use blake2::{Blake2b512, Digest};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::config::AppConfig;

pub type ChecksumCache = Mutex<HashMap<String, Blake2b512>>;

pub fn upload_file_path(config: &AppConfig, upload_id: &str) -> PathBuf {
    Path::new(&config.storage_path).join("uploads").join(upload_id)
}

pub fn final_file_path(config: &AppConfig, file_hash: &str) -> PathBuf {
    Path::new(&config.storage_path).join("files").join(file_hash)
}

pub fn checksum_hex(hasher: &Blake2b512) -> String {
    hasher.clone().finalize().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hasher_from_prefix(path: &Path, len: u64) -> std::io::Result<Blake2b512> {
    let mut file = File::open(path)?;
    let mut hasher = Blake2b512::new();
    let mut buf = [0u8; 64 * 1024];
    let mut remaining = len;

    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        file.read_exact(&mut buf[..to_read])?;
        hasher.update(&buf[..to_read]);
        remaining -= to_read as u64;
    }

    Ok(hasher)
}

pub fn ensure_upload_file(path: &Path, file_size: Option<i64>) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        let file = File::create(path)?;
        if let Some(size) = file_size {
            file.set_len(size as u64)?;
        }
    }

    OpenOptions::new().write(true).open(path)
}

pub struct UploadMetadataFields {
    pub file_name: String,
    pub checksum: Option<String>,
    pub mime_type: String,
    pub file_dir: String,
}

impl TryFrom<HashMap<String, Option<String>>> for UploadMetadataFields {
    type Error = String;

    fn try_from(mut value: HashMap<String, Option<String>>) -> Result<Self, Self::Error> {
        let file_name = value
            .remove("file_name")
            .flatten()
            .ok_or("Missing required metadata: file_name")?;

        let checksum = value.remove("checksum").flatten();

        let mime_type = value
            .remove("mime_type")
            .flatten()
            .ok_or("Missing required metadata: mime_type")?;

        let file_dir = value
            .remove("file_dir")
            .flatten()
            .unwrap_or(String::from("/"));

        Ok(Self {
            file_name,
            checksum,
            mime_type,
            file_dir,
        })
    }
}

pub fn decode_metadata(input: &str) -> Result<HashMap<String, Option<String>>, String> {
    let mut metadata = HashMap::new();

    for pair in input.split(",") {
        let pair = pair.trim();

        if pair.is_empty() {
            return Err("Empty metadata pair".into());
        }

        let (key, value) = match pair.split_once(" ") {
            Some((key, encoded)) => {
                if !key.is_ascii() {
                    return Err(format!("Metadata key {key} is not ASCII"));
                }

                let decoded = STANDARD
                    .decode(encoded)
                    .map_err(|_| format!("Invalid base64 for key '{key}'"))?;

                let value = String::from_utf8(decoded)
                    .map_err(|_| format!("Value for '{key}' is not UTF-8"))?;

                (key.to_string(), Some(value))
            }
            None => {
                if !pair.is_ascii() {
                    return Err(format!("Metadata key {pair} is not ASCII"));
                }

                (pair.to_string(), None)
            }
        };

        metadata.insert(key, value);
    }

    Ok(metadata)
}
