use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::time::Duration;

use actix_web::web;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::AppConfig;

use super::github;
use super::restart::{exe_path, old_exe_path};
use super::sentinel;
use super::state::{InstallPhase, ReleaseInfo, UpdateState};

const MAX_ASSET_BYTES: u64 = 200 * 1024 * 1024;
const MAX_SUMS_BYTES: u64 = 64 * 1024;

const DOWNLOAD_BUF: usize = 64 * 1024;
const PROGRESS_STEP: u64 = 256 * 1024;

pub fn spawn_install(state: web::Data<UpdateState>, config: AppConfig, release: ReleaseInfo) {
    let worker_state = state.clone();
    let spawned = std::thread::Builder::new()
        .name("update-install".into())
        .spawn(move || {
            let version = release.version.clone();
            match run_install(&worker_state, &config, release) {
                Ok(()) => {
                    worker_state.set_phase(InstallPhase::Ready);
                    println!("Installed v{version}; restart to apply.");
                }
                Err(err) => {
                    eprintln!("WARN: update install failed: {err}");
                    worker_state.fail_install(err);
                }
            }
        });

    if let Err(err) = spawned {
        let message = format!("Could not start the installer: {err}");
        eprintln!("WARN: {message}");
        state.fail_install(message);
    }
}

fn run_install(
    state: &UpdateState,
    config: &AppConfig,
    release: ReleaseInfo,
) -> Result<(), String> {
    let exe = exe_path();
    let dir = exe
        .parent()
        .ok_or_else(|| "Cannot determine the install directory".to_string())?
        .to_path_buf();

    preflight(&dir)?;

    // Re-validate rather than trusting what the check cached.
    github::validate_download_url(&release.asset_url, config)?;
    github::validate_download_url(&release.sums_url, config)?;

    let agent = github::agent(config);

    let sums = agent
        .get(&release.sums_url)
        .call()
        .map_err(|err| format!("Could not fetch {}: {err}", github::SUMS_ASSET))?
        .body_mut()
        .with_config()
        .limit(MAX_SUMS_BYTES)
        .read_to_string()
        .map_err(|err| format!("Could not read {}: {err}", github::SUMS_ASSET))?;

    let expected = find_checksum(&sums, &release.asset_name).ok_or_else(|| {
        format!(
            "{} has no checksum for {}",
            github::SUMS_ASSET,
            release.asset_name
        )
    })?;

    // Same directory as the executable, so the swap is a rename within one
    // volume rather than a copy across two.
    let temp = dir.join(format!(".fileserve-update-{}.tmp", Uuid::new_v4()));

    let digest = match download(state, &agent, &release, &temp) {
        Ok(digest) => digest,
        Err(err) => {
            let _ = fs::remove_file(&temp);
            return Err(err);
        }
    };

    state.set_phase(InstallPhase::Verifying);
    if !digest.eq_ignore_ascii_case(expected) {
        let _ = fs::remove_file(&temp);
        return Err(format!(
            "Checksum mismatch for {} — the download was rejected",
            release.asset_name
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(err) = fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)) {
            let _ = fs::remove_file(&temp);
            return Err(format!("Could not mark the new binary executable: {err}"));
        }
    }

    state.set_phase(InstallPhase::Applying);

    sentinel::write(&release.version)?;

    if let Err(err) = swap(&exe, &temp) {
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(sentinel::record_path());
        return Err(err);
    }

    Ok(())
}

/// Checked before downloading, so a root-owned or read-only install directory
/// fails fast instead of after 20 MB.
fn preflight(dir: &Path) -> Result<(), String> {
    let probe = dir.join(format!(".fileserve-probe-{}", Uuid::new_v4()));
    File::create(&probe).map_err(|err| {
        format!(
            "Cannot write to {} — the server needs write access to its own \
             directory to update itself ({err})",
            dir.display()
        )
    })?;
    let _ = fs::remove_file(&probe);
    Ok(())
}

/// Streams the asset to `temp`, hashing as it goes, and returns the hex digest.
fn download(
    state: &UpdateState,
    agent: &ureq::Agent,
    release: &ReleaseInfo,
    temp: &Path,
) -> Result<String, String> {
    let mut response = agent
        .get(&release.asset_url)
        .call()
        .map_err(|err| format!("Could not download {}: {err}", release.asset_name))?;

    let mut reader = response.body_mut().as_reader();
    let file =
        File::create(temp).map_err(|err| format!("Could not create the download file: {err}"))?;
    let mut writer = BufWriter::new(file);

    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; DOWNLOAD_BUF];
    let mut written: u64 = 0;
    let mut since_report: u64 = 0;

    loop {
        let read = reader
            .read(&mut buf)
            .map_err(|err| format!("The download was interrupted: {err}"))?;
        if read == 0 {
            break;
        }

        written += read as u64;
        if written > MAX_ASSET_BYTES {
            return Err(format!(
                "{} is larger than the {} MiB limit",
                release.asset_name,
                MAX_ASSET_BYTES / (1024 * 1024)
            ));
        }

        hasher.update(&buf[..read]);
        writer
            .write_all(&buf[..read])
            .map_err(|err| format!("Could not write the download: {err}"))?;

        since_report += read as u64;
        if since_report >= PROGRESS_STEP {
            state.set_progress(written, release.asset_size.max(written));
            since_report = 0;
        }
    }

    let file = writer
        .into_inner()
        .map_err(|err| format!("Could not flush the download: {err}"))?;
    file.sync_all()
        .map_err(|err| format!("Could not flush the download to disk: {err}"))?;

    state.set_progress(written, release.asset_size.max(written));
    Ok(hex(&hasher.finalize()))
}

/// A running executable cannot be deleted or written to on Windows, but it
/// *can* be renamed — hence moving the current one aside rather than writing
/// over it. It stays as `.old` until the replacement proves it can start.
fn swap(exe: &Path, temp: &Path) -> Result<(), String> {
    let old = old_exe_path();
    let _ = fs::remove_file(&old);

    fs::rename(exe, &old)
        .map_err(|err| format!("Could not move the running binary aside: {err}"))?;

    if let Err(err) = rename_with_retry(temp, exe) {
        // Never leave the machine without an executable.
        let _ = fs::rename(&old, exe);
        return Err(format!(
            "Could not install the new binary, so the previous one was put back: {err}"
        ));
    }

    Ok(())
}

/// Windows antivirus routinely holds a brief lock on a newly written
/// executable, which surfaces as a spurious rename failure.
fn rename_with_retry(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut last = None;
    for attempt in 0..3u32 {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(err) => {
                last = Some(err);
                std::thread::sleep(Duration::from_millis(200 * u64::from(attempt + 1)));
            }
        }
    }
    Err(last.expect("loop runs at least once"))
}

fn find_checksum<'a>(manifest: &'a str, asset_name: &str) -> Option<&'a str> {
    manifest.lines().find_map(|line| {
        let (hash, rest) = line.trim().split_once(char::is_whitespace)?;
        let name = rest.trim_start().trim_start_matches('*');
        (name.trim_end() == asset_name
            && hash.len() == 64
            && hash.chars().all(|c| c.is_ascii_hexdigit()))
        .then_some(hash)
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::find_checksum;

    // Line one is text mode (Linux runner), line two binary mode (Windows).
    const SUMS: &str = "\
1111111111111111111111111111111111111111111111111111111111111111  fileserve-rs-v0.1.2-x86_64-unknown-linux-gnu
2222222222222222222222222222222222222222222222222222222222222222 *fileserve-rs-v0.1.2-x86_64-pc-windows-msvc.exe
";

    #[test]
    fn finds_the_matching_asset() {
        let expected = "2".repeat(64);
        assert_eq!(
            find_checksum(SUMS, "fileserve-rs-v0.1.2-x86_64-pc-windows-msvc.exe"),
            Some(expected.as_str())
        );
    }

    #[test]
    fn finds_a_text_mode_entry() {
        let expected = "1".repeat(64);
        assert_eq!(
            find_checksum(SUMS, "fileserve-rs-v0.1.2-x86_64-unknown-linux-gnu"),
            Some(expected.as_str())
        );
    }

    #[test]
    fn ignores_a_missing_asset() {
        assert_eq!(find_checksum(SUMS, "fileserve-rs-v0.1.2-aarch64-apple-darwin"), None);
    }

    #[test]
    fn rejects_a_malformed_digest() {
        assert_eq!(find_checksum("abc  some-asset", "some-asset"), None);
    }
}
