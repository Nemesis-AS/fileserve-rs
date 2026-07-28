use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::restart::{exe_path, old_exe_path, with_suffix};

const RECORD_DIR: &str = "data";
const RECORD_FILE: &str = "update-pending.json";

/// Reaching `main()` this many times without confirming healthy means a crash
/// loop. Two, because the first boot is the legitimate one.
const MAX_BOOT_ATTEMPTS: u32 = 2;

/// Written before the binaries are swapped, so a crash mid-swap still leaves
/// the next boot something to reason about.
#[derive(Serialize, Deserialize)]
pub struct PendingUpdate {
    pub version: String,
    pub previous_version: String,
    pub exe: PathBuf,
    pub old: PathBuf,
    pub installed_at: DateTime<Utc>,
    #[serde(default)]
    pub boot_attempts: u32,
}

pub fn record_path() -> PathBuf {
    Path::new(RECORD_DIR).join(RECORD_FILE)
}

pub fn write(version: &str) -> Result<(), String> {
    let pending = PendingUpdate {
        version: version.to_string(),
        previous_version: env!("CARGO_PKG_VERSION").to_string(),
        exe: exe_path(),
        old: old_exe_path(),
        installed_at: Utc::now(),
        boot_attempts: 0,
    };
    save(&pending)
}

fn save(pending: &PendingUpdate) -> Result<(), String> {
    fs::create_dir_all(RECORD_DIR)
        .map_err(|err| format!("Could not create the data directory: {err}"))?;
    let json = serde_json::to_string_pretty(pending)
        .map_err(|err| format!("Could not serialize the update record: {err}"))?;
    fs::write(record_path(), json)
        .map_err(|err| format!("Could not write the update record: {err}"))
}

fn load() -> Option<PendingUpdate> {
    let raw = fs::read_to_string(record_path()).ok()?;
    match serde_json::from_str(&raw) {
        Ok(pending) => Some(pending),
        Err(err) => {
            eprintln!("WARN: ignoring unreadable update record: {err}");
            None
        }
    }
}

pub enum BootOutcome {
    Nothing,
    OnTrial { version: String },
    /// The previous binary is back; this process should exit so it gets started.
    RolledBack { restored: String },
}

/// Called at the top of `main`. The half of the safety net that works under a
/// service manager, where nothing can supervise the handover.
pub fn resolve_pending() -> BootOutcome {
    let Some(mut pending) = load() else {
        cleanup_stale();
        return BootOutcome::Nothing;
    };

    if pending.version != env!("CARGO_PKG_VERSION") {
        eprintln!(
            "WARN: discarding update record for v{} (running v{})",
            pending.version,
            env!("CARGO_PKG_VERSION")
        );
        let _ = fs::remove_file(record_path());
        return BootOutcome::Nothing;
    }

    pending.boot_attempts += 1;

    if pending.boot_attempts >= MAX_BOOT_ATTEMPTS {
        eprintln!(
            "WARN: v{} has started {} times without becoming healthy — rolling back to v{}.",
            pending.version, pending.boot_attempts, pending.previous_version
        );
        return match roll_back(&pending) {
            Ok(()) => BootOutcome::RolledBack {
                restored: pending.previous_version.clone(),
            },
            Err(err) => {
                eprintln!("WARN: rollback failed: {err}");
                eprintln!(
                    "WARN: restore it by hand — rename {} back to {}",
                    pending.old.display(),
                    pending.exe.display()
                );
                let _ = fs::remove_file(record_path());
                BootOutcome::Nothing
            }
        };
    }

    if let Err(err) = save(&pending) {
        eprintln!("WARN: could not update the boot counter: {err}");
    }
    BootOutcome::OnTrial {
        version: pending.version.clone(),
    }
}

pub fn roll_back_pending() -> Result<String, String> {
    let pending = load().ok_or_else(|| "there is no update on trial".to_string())?;
    roll_back(&pending)?;
    Ok(pending.previous_version)
}

/// The failed binary is renamed aside, not deleted — on Windows it may still be
/// a running image.
fn roll_back(pending: &PendingUpdate) -> Result<(), String> {
    if !pending.old.exists() {
        return Err(format!("{} is missing", pending.old.display()));
    }

    let failed = with_suffix(&pending.exe, ".failed");
    let _ = fs::remove_file(&failed);
    fs::rename(&pending.exe, &failed)
        .map_err(|err| format!("could not move the failed binary aside: {err}"))?;

    if let Err(err) = fs::rename(&pending.old, &pending.exe) {
        let _ = fs::rename(&failed, &pending.exe);
        return Err(format!("could not restore the previous binary: {err}"));
    }

    let _ = fs::remove_file(record_path());
    Ok(())
}

/// Called on a delay after binding, so it means "started *and stayed up*".
pub fn confirm() {
    // Only ever retire a record describing *this* binary. One naming another
    // version belongs to an install that happened since we booted, and it is
    // the only thing standing between a bad build and an unrecoverable server.
    match load() {
        Some(pending) if pending.version == env!("CARGO_PKG_VERSION") => {}
        Some(pending) => {
            println!(
                "Leaving the pending update to v{} alone; it is not this build.",
                pending.version
            );
            return;
        }
        None => return,
    }

    if let Err(err) = fs::remove_file(record_path()) {
        eprintln!("WARN: could not remove the update record: {err}");
        return;
    }

    println!("Update to v{} confirmed healthy.", env!("CARGO_PKG_VERSION"));
    cleanup_stale();
}

/// Deleting `.old` can fail on Windows while the process that ran it is alive;
/// the next boot retries, so failures aren't worth reporting.
pub fn cleanup_stale() {
    let exe = exe_path();

    for leftover in [with_suffix(&exe, ".old"), with_suffix(&exe, ".failed")] {
        if leftover.exists() && fs::remove_file(&leftover).is_ok() {
            println!("Removed {}", leftover.display());
        }
    }

    let Some(dir) = exe.parent() else { return };
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(".fileserve-update-") && name.ends_with(".tmp") {
            let _ = fs::remove_file(entry.path());
        }
    }
}
