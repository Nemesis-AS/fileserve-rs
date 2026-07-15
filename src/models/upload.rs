use chrono::{DateTime, Utc};

pub struct Upload {
    id: String,
    path: String,
    expected_size: u64,
    filename: String,
    mimetype: String,
    owner: String,
    object_hash: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}
