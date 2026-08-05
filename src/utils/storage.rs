use std::fs;

use sqlx::{Pool, Sqlite};

use crate::utils::tus::final_file_path;

/// Removes the blob backing `checksum`, but only when no `files` row still
/// references it.
///
/// Storage is content-addressed, so one blob can back several owners' files:
/// two users uploading identical bytes share it. Unlinking on the strength of
/// one owner's delete would take the other's file with it.
///
/// The count deliberately does **not** filter on `deleted_at`. A trashed file
/// still references its bytes and must still be restorable, so a row in the
/// trash keeps the blob alive.
///
/// Returns whether the blob was removed. Filesystem errors are logged and
/// reported as `false` rather than propagated: a blob that outlives its rows
/// wastes disk, which is recoverable, whereas failing the caller over it is not
/// worth the disruption.
pub async fn release_blob_if_unreferenced(
    pool: &Pool<Sqlite>,
    storage_path: &str,
    checksum: &str,
) -> Result<bool, sqlx::Error> {
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE checksum = ?")
        .bind(checksum)
        .fetch_one(pool)
        .await?;

    if remaining > 0 {
        return Ok(false);
    }

    let path = final_file_path(storage_path, checksum);
    match actix_web::web::block(move || fs::remove_file(&path)).await {
        Ok(Ok(())) => Ok(true),
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Ok(Err(e)) => {
            eprintln!("WARN: failed to remove blob {checksum}: {e}");
            Ok(false)
        }
        Err(e) => {
            eprintln!("WARN: blob removal task failed for {checksum}: {e}");
            Ok(false)
        }
    }
}
