use std::{
    env,
    fs::{self, File, Permissions},
    io,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
};

use chrono::Utc;
use color_eyre::eyre::{Context, Result};
use displaydoc::Display;
use log::{debug, info, trace};
use serde_derive::Deserialize;
use thiserror::Error;

use self::UpdateSelfError as E;
use crate::{
    opts::UpdateSelfOptions,
    tasks::{task::TaskStatus, ResolveEnv},
};

#[derive(Debug, Deserialize)]
struct GitHubReleaseJsonResponse {
    tag_name: String,
}

// Name user agent after the app, e.g. up-rs/1.2.3.
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl ResolveEnv for UpdateSelfOptions {}

/// Downloads the latest version of the binary from the specified URL and
/// replaces the current executable path with it.
pub(crate) fn run(opts: &UpdateSelfOptions) -> Result<TaskStatus> {
    let up_path = env::current_exe()?.canonicalize().unwrap();

    // If the current binary's location is where it was originally compiled, assume it is a dev
    // build, and thus skip the update.
    if !opts.always_update && up_path.starts_with(env!("CARGO_MANIFEST_DIR")) {
        debug!("Skipping up-rs update, current version '{up_path:?}' is a dev build.",);
        return Ok(TaskStatus::Skipped);
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    trace!("Self update opts: {opts:?}");
    if opts.url == crate::opts::SELF_UPDATE_URL {
        let latest_github_release = client
            .get(crate::opts::LATEST_RELEASE_URL)
            .send()?
            .error_for_status()?
            .json::<GitHubReleaseJsonResponse>()?;
        trace!("latest_github_release: {latest_github_release:?}");
        let latest_github_release = latest_github_release.tag_name;
        if semver::Version::parse(&latest_github_release)?
            <= semver::Version::parse(CURRENT_VERSION)?
        {
            debug!(
                "Skipping up-rs update, current version '{CURRENT_VERSION}' is not older than latest GitHub version '{latest_github_release}'",
            );
            return Ok(TaskStatus::Skipped);
        }
        trace!("Updating up-rs from '{CURRENT_VERSION}' to '{latest_github_release}'",);
    }

    let temp_dir = env::temp_dir();
    let temp_path = &temp_dir.join(format!("up_rs-{}", Utc::now().to_rfc3339()));

    trace!("Downloading url {url} to path {up_path:?}", url = &opts.url,);

    trace!("Using temporary path: {temp_path:?}");
    let mut response = reqwest::blocking::get(&opts.url)?.error_for_status()?;

    fs::create_dir_all(&temp_dir).with_context(|| E::CreateDir { path: temp_dir })?;
    let mut dest = File::create(&temp_path).with_context(|| E::CreateFile {
        path: temp_path.clone(),
    })?;
    io::copy(&mut response, &mut dest).context(E::Copy {})?;

    let permissions = Permissions::from_mode(0o755);
    fs::set_permissions(&temp_path, permissions).with_context(|| E::SetPermissions {
        path: temp_path.clone(),
    })?;

    let output = Command::new(temp_path).arg("--version").output()?;
    let new_version = String::from_utf8_lossy(&output.stdout);
    let new_version = new_version
        .trim()
        .trim_start_matches(concat!(env!("CARGO_PKG_NAME"), " "));
    if semver::Version::parse(new_version)? > semver::Version::parse(CURRENT_VERSION)? {
        info!("Updating up-rs from '{CURRENT_VERSION}' to '{new_version}'",);
        fs::rename(&temp_path, &up_path).with_context(|| E::Rename {
            from: temp_path.clone(),
            to: up_path.clone(),
        })?;
        Ok(TaskStatus::Passed)
    } else {
        debug!(
            "Skipping up-rs update, current version '{CURRENT_VERSION}' and new version '{new_version}'",
        );
        Ok(TaskStatus::Skipped)
    }
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum UpdateSelfError {
    /// Failed to create directory '{path}'
    CreateDir { path: PathBuf },
    /// Failed to create file '{path}'
    CreateFile { path: PathBuf },
    /// Failed to copy to destination file.
    Copy,
    /// Failed to set permissions for {path}.
    SetPermissions { path: PathBuf },
    /// Failed to rename {from} to {to}.
    Rename { from: PathBuf, to: PathBuf },
}
