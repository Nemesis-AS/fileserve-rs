use serde::Serialize;
use sqlx::FromRow;

/// Column list shared by every `files` query that hydrates a [`FileRecord`].
/// `SELECT *` would also pull `file_dir`, which no consumer needs yet.
pub const FILE_COLUMNS: &str =
    "id, file_name, mime_type, file_size, checksum, owner_uname, created_at, deleted_at";

#[derive(FromRow, Serialize)]
pub struct FileRecord {
    pub id: String,
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub checksum: String,
    pub owner_uname: String,
    /// Timestamps travel as strings, not chrono types: `created_at` carries
    /// SQLite's `date()` default (`YYYY-MM-DD`) while `deleted_at` is written as
    /// a full ISO-8601 instant, so the two columns hold different shapes and
    /// both parse fine as JS `Date`s on the client.
    pub created_at: Option<String>,
    /// Non-null means the file is in the trash.
    pub deleted_at: Option<String>,
}
