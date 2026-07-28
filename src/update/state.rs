use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<DateTime<Utc>>,
    pub asset_name: String,
    pub asset_size: u64,
    #[serde(skip)]
    pub asset_url: String,
    #[serde(skip)]
    pub sums_url: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    #[default]
    Idle,
    Downloading,
    Verifying,
    Applying,
    /// Installed on disk; takes effect on the next restart.
    Ready,
    Failed,
}

impl InstallPhase {
    pub fn is_busy(self) -> bool {
        matches!(self, Self::Downloading | Self::Verifying | Self::Applying)
    }
}

#[derive(Clone, Default, Serialize)]
pub struct InstallState {
    pub phase: InstallPhase,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Default, Serialize)]
pub struct UpdateStateData {
    pub last_checked: Option<DateTime<Utc>>,
    pub checking: bool,
    pub check_error: Option<String>,
    pub latest: Option<ReleaseInfo>,
    pub install: InstallState,
}

#[derive(Default)]
pub struct UpdateState {
    inner: Mutex<UpdateStateData>,
}

impl UpdateState {
    fn lock(&self) -> MutexGuard<'_, UpdateStateData> {
        self.inner.lock().expect("update state lock poisoned")
    }

    pub fn snapshot(&self) -> UpdateStateData {
        self.lock().clone()
    }

    pub fn begin_check(&self) -> bool {
        let mut data = self.lock();
        if data.checking {
            return false;
        }
        data.checking = true;
        true
    }

    pub fn finish_check(&self, result: Result<Option<ReleaseInfo>, String>) {
        let mut data = self.lock();
        data.checking = false;
        data.last_checked = Some(Utc::now());
        match result {
            Ok(latest) => {
                data.check_error = None;
                data.latest = latest;
            }
            // Keep the last known release on error: a transient network blip
            // shouldn't retract an update the admin was already being offered.
            Err(err) => data.check_error = Some(err),
        }
    }

    pub fn try_begin_install(&self, version: &str) -> Result<ReleaseInfo, InstallRejected> {
        let mut data = self.lock();

        if data.install.phase.is_busy() {
            return Err(InstallRejected::InProgress);
        }
        if data.install.phase == InstallPhase::Ready {
            return Err(InstallRejected::AlreadyInstalled);
        }

        let release = match &data.latest {
            Some(release) if release.version == version => release.clone(),
            Some(_) => return Err(InstallRejected::VersionMismatch),
            None => return Err(InstallRejected::NothingAvailable),
        };

        data.install = InstallState {
            phase: InstallPhase::Downloading,
            downloaded_bytes: 0,
            total_bytes: release.asset_size,
            version: Some(release.version.clone()),
            error: None,
        };
        Ok(release)
    }

    pub fn install_snapshot(&self) -> InstallState {
        self.lock().install.clone()
    }

    pub fn set_progress(&self, downloaded: u64, total: u64) {
        let mut data = self.lock();
        data.install.downloaded_bytes = downloaded;
        data.install.total_bytes = total;
    }

    pub fn set_phase(&self, phase: InstallPhase) {
        self.lock().install.phase = phase;
    }

    pub fn fail_install(&self, error: String) {
        let mut data = self.lock();
        data.install.phase = InstallPhase::Failed;
        data.install.error = Some(error);
    }

}

pub enum InstallRejected {
    InProgress,
    AlreadyInstalled,
    /// The client asked for a version that is no longer the latest — it was
    /// looking at a stale page. Better to refuse than to install something the
    /// admin never saw.
    VersionMismatch,
    NothingAvailable,
}

impl InstallRejected {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InProgress => "An update is already being installed",
            Self::AlreadyInstalled => {
                "An update is already installed and waiting for a restart"
            }
            Self::VersionMismatch => {
                "That version is no longer the latest release — check for updates again"
            }
            Self::NothingAvailable => "No update is available to install",
        }
    }
}
