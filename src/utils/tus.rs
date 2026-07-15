use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::HashMap;

pub struct UploadMetadataFields {
    pub file_name: String,
    pub checksum: String,
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

        let checksum = value
            .remove("checksum")
            .flatten()
            .ok_or("Missing required metadata: checksum")?;

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
