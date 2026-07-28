mod github;
mod install;
mod restart;
mod sentinel;
mod state;

pub use github::{run_check, spawn_checker};
pub use install::spawn_install;
pub use restart::{RestartSignal, capture_exe_path, supervise_successor};
pub use sentinel::{BootOutcome, confirm, resolve_pending};
pub use state::{UpdateState, UpdateStateData};
