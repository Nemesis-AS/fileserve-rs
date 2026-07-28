use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use actix_web::dev::ServerHandle;
use ureq::Agent;

use super::sentinel;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(500);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

/// Captured at startup because `current_exe()` resolves `/proc/self/exe` on
/// Linux — once an update renames the running binary aside it starts reporting
/// `…exe.old`, and relaunching that would silently start the old version.
static EXE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn capture_exe_path() {
    let path = std::env::current_exe().expect("Failed to resolve current executable path!");
    let _ = EXE_PATH.set(path);
}

pub fn exe_path() -> PathBuf {
    EXE_PATH
        .get()
        .cloned()
        .expect("exe path was not captured at startup")
}

/// Appends to the whole file name — not `with_extension`, which would turn
/// `fileserve-rs.exe` into `fileserve-rs.old`.
pub fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("fileserve-rs"));
    name.push(suffix);
    path.with_file_name(name)
}

pub fn old_exe_path() -> PathBuf {
    with_suffix(&exe_path(), ".old")
}

/// The handle only exists after `run()`, but the app factory is built before
/// that — hence the `OnceLock`.
#[derive(Default)]
pub struct RestartSignal {
    handle: OnceLock<ServerHandle>,
    requested: AtomicBool,
}

impl RestartSignal {
    pub fn set_handle(&self, handle: ServerHandle) {
        let _ = self.handle.set(handle);
    }

    pub fn handle(&self) -> Option<ServerHandle> {
        self.handle.get().cloned()
    }

    pub fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);
    }

    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }
}

/// The working directory is passed explicitly because the database and storage
/// paths are relative — a successor started elsewhere would come up empty.
pub fn spawn_successor() -> std::io::Result<Child> {
    let exe = exe_path();
    let cwd = std::env::current_dir()?;
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();

    let mut command = Command::new(&exe);
    command.args(args).current_dir(cwd).stdin(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Not DETACHED_PROCESS: that leaves the successor with no console and
        // silently discards everything it prints. stdout/stderr stay inherited.
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }

    command.spawn()
}

/// Called from `main` after the server has stopped, so the port is free. This
/// is what catches a binary too broken to reach `main()` at all.
pub fn supervise_successor(port: u16) {
    let mut child = match spawn_successor() {
        Ok(child) => child,
        Err(err) => {
            eprintln!("WARN: could not start the new version: {err}");
            attempt_rollback();
            return;
        }
    };

    println!("Started the new version; waiting for it to come up…");

    let agent = health_agent();
    let deadline = Instant::now() + STARTUP_TIMEOUT;

    loop {
        if is_healthy(&agent, port) {
            println!("New version is up.");
            return;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("WARN: the new version exited immediately ({status}).");
                break;
            }
            Ok(None) => {}
            Err(err) => {
                eprintln!("WARN: lost track of the new version: {err}");
                break;
            }
        }

        if Instant::now() >= deadline {
            eprintln!(
                "WARN: the new version did not respond within {}s.",
                STARTUP_TIMEOUT.as_secs()
            );
            let _ = child.kill();
            let _ = child.wait();
            break;
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    attempt_rollback();
}

fn attempt_rollback() {
    match sentinel::roll_back_pending() {
        Ok(restored) => {
            println!("Rolled back to v{restored}. Starting it…");
            if let Err(err) = spawn_successor() {
                eprintln!("WARN: could not start the restored version: {err}");
                eprintln!("WARN: start the server manually.");
            }
        }
        Err(err) => {
            eprintln!("WARN: could not roll back: {err}");
            eprintln!("WARN: the server is not running. Start it manually.");
        }
    }
}

fn health_agent() -> Agent {
    let config = Agent::config_builder()
        .timeout_global(Some(HEALTH_TIMEOUT))
        .build();
    Agent::new_with_config(config)
}

fn is_healthy(agent: &Agent, port: u16) -> bool {
    agent
        .get(&format!("http://127.0.0.1:{port}/api/v1/health"))
        .call()
        .is_ok()
}
