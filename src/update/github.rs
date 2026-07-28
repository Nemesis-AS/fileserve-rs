use std::time::Duration;

use chrono::{DateTime, Utc};
use semver::Version;
use serde::Deserialize;
use ureq::Agent;

use crate::config::AppConfig;
use crate::update::state::{ReleaseInfo, UpdateState};

const USER_AGENT: &str = concat!("fileserve-rs/", env!("CARGO_PKG_VERSION"));

pub const SUMS_ASSET: &str = "SHA256SUMS";

const GITHUB_HOSTS: &[&str] = &["github.com", "api.github.com"];
const GITHUB_ASSET_SUFFIX: &str = ".githubusercontent.com";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The slice of GitHub's release JSON this actually reads.
#[derive(Deserialize)]
pub struct GhRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
pub struct GhAsset {
    pub name: String,
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
}

pub fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("crate version is not valid semver")
}

pub fn expected_asset_name(tag_name: &str) -> String {
    format!(
        "fileserve-rs-{}-{}{}",
        tag_name,
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    )
}

pub fn agent(config: &AppConfig) -> Agent {
    let cfg = Agent::config_builder()
        .user_agent(USER_AGENT)
        .https_only(config.update_api_base.starts_with("https://"))
        .max_redirects(5)
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build();
    Agent::new_with_config(cfg)
}

pub fn validate_download_url(url: &str, config: &AppConfig) -> Result<(), String> {
    let host = host_of(url).ok_or_else(|| format!("Unusable download URL: {url}"))?;

    let allowed = GITHUB_HOSTS.contains(&host)
        || host.ends_with(GITHUB_ASSET_SUFFIX)
        || host_of(&config.update_api_base).is_some_and(|configured| configured == host);

    if allowed {
        Ok(())
    } else {
        Err(format!("Refusing to download from untrusted host '{host}'"))
    }
}

fn host_of(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.contains('@') || authority.is_empty() {
        return None;
    }
    authority.split(':').next().filter(|h| !h.is_empty())
}

pub fn fetch_latest(config: &AppConfig) -> Result<GhRelease, String> {
    let url = format!(
        "{}/repos/{}/releases/latest",
        config.update_api_base, config.update_repo
    );

    let mut response = agent(config)
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .map_err(|err| describe(err, &config.update_repo))?;

    response
        .body_mut()
        .read_json::<GhRelease>()
        .map_err(|err| format!("Could not read the release from GitHub: {err}"))
}

fn describe(err: ureq::Error, repo: &str) -> String {
    match err {
        ureq::Error::StatusCode(404) => {
            format!("{repo} has no published releases yet")
        }
        ureq::Error::StatusCode(403) => {
            "GitHub rejected the request — most likely the unauthenticated rate limit \
             (60 requests per hour). Try again later."
                .to_string()
        }
        ureq::Error::StatusCode(code) => format!("GitHub returned HTTP {code}"),
        other => format!("Could not reach GitHub: {other}"),
    }
}

pub fn resolve(release: &GhRelease, config: &AppConfig) -> Result<ReleaseInfo, String> {
    let version = Version::parse(release.tag_name.trim_start_matches('v'))
        .map_err(|err| format!("Release tag '{}' is not valid semver: {err}", release.tag_name))?;

    let wanted = expected_asset_name(&release.tag_name);
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == wanted)
        .ok_or_else(|| {
            format!(
                "Release {} has no download for this platform ({})",
                release.tag_name,
                env!("BUILD_TARGET")
            )
        })?;

    let sums = release
        .assets
        .iter()
        .find(|asset| asset.name == SUMS_ASSET)
        .ok_or_else(|| {
            format!(
                "Release {} is missing its {SUMS_ASSET} checksum manifest",
                release.tag_name
            )
        })?;

    validate_download_url(&asset.browser_download_url, config)?;
    validate_download_url(&sums.browser_download_url, config)?;

    Ok(ReleaseInfo {
        version: version.to_string(),
        tag_name: release.tag_name.clone(),
        name: release.name.clone(),
        body: release.body.clone(),
        html_url: release.html_url.clone(),
        published_at: release.published_at,
        asset_name: asset.name.clone(),
        asset_size: asset.size,
        asset_url: asset.browser_download_url.clone(),
        sums_url: sums.browser_download_url.clone(),
    })
}

pub fn run_check(state: &UpdateState, config: &AppConfig) {
    if !state.begin_check() {
        return;
    }

    let result = check_once(config);
    match &result {
        Ok(Some(release)) => println!("Update available: {}", release.version),
        Ok(None) => {}
        Err(err) => eprintln!("WARN: update check failed: {err}"),
    }
    state.finish_check(result);
}

fn check_once(config: &AppConfig) -> Result<Option<ReleaseInfo>, String> {
    let release = fetch_latest(config)?;

    if release.draft || release.prerelease {
        return Ok(None);
    }

    let latest = Version::parse(release.tag_name.trim_start_matches('v'))
        .map_err(|err| format!("Release tag '{}' is not valid semver: {err}", release.tag_name))?;
    if latest <= current_version() {
        return Ok(None);
    }

    resolve(&release, config).map(Some)
}

pub fn spawn_checker(state: actix_web::web::Data<UpdateState>, config: AppConfig) {
    if !config.self_update_enabled {
        println!("Self-update is disabled; skipping update checks.");
        return;
    }

    let spawned = std::thread::Builder::new()
        .name("update-checker".into())
        .spawn(move || {
            loop {
                run_check(&state, &config);
                std::thread::sleep(Duration::from_secs(24 * 60 * 60));
            }
        });

    if let Err(err) = spawned {
        eprintln!("WARN: could not start the update checker: {err}");
    }
}
