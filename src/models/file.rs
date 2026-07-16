use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct FileRecord {
    pub id: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub checksum: String,
    pub owner_uname: String,
}
