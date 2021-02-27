use std::{
    env,
    fs::{self, File, Permissions},
    io,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
};

use anyhow::{Context, Result};
use chrono::Utc;
use displaydoc::Display;
use log::{debug, info, trace};
use serde_derive::Deserialize;
use thiserror::Error;

use self::UpdateSelfError as E;
use crate::{args::UpdateSelfOptions, tasks::ResolveEnv};

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
pub(crate) fn run(opts: &UpdateSelfOptions) -> Result<()> {
    let up_path = env::current_exe()?.canonicalize().unwrap();

    // If the current binary's location is where it was originally compiled, assume it is a dev
    // build, and thus skip the update.
    if !opts.always_update && up_path.starts_with(env!("CARGO_MANIFEST_DIR")) {
        info!(
            "Skipping up-rs update, current version '{}' is a dev build.",
            &up_path.display(),
        );
        return Ok(());
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    if opts.url == crate::args::SELF_UPDATE_URL {
        let latest_github_release = client
            .get(crate::args::LATEST_RELEASE_URL)
            .send()?
            .error_for_status()?
            .json::<GitHubReleaseJsonResponse>()?;
        trace!("latest_github_release: {:?}", latest_github_release,);
        let latest_github_release = latest_github_release.tag_name;
        if CURRENT_VERSION == latest_github_release {
            info!(
                "Skipping up-rs update, current version '{}' is latest GitHub version '{:?}'",
                CURRENT_VERSION, &latest_github_release,
            );
            return Ok(());
        }
    }

    let temp_dir = env::temp_dir();
    let temp_path = &temp_dir.join(format!("up_rs-{}", Utc::now().to_rfc3339()));

    debug!(
        "Downloading url {} to path {}",
        &opts.url,
        up_path.display()
    );

    debug!("Using temporary path: {}", temp_path.display());
    let mut response = reqwest::blocking::get(&opts.url)?.error_for_status()?;

    fs::create_dir_all(&temp_dir).with_context(|| E::CreateDir { path: temp_dir })?;
    let mut dest = File::create(&temp_path).with_context(|| E::CreateFile {
        path: temp_path.to_path_buf(),
    })?;
    io::copy(&mut response, &mut dest).context(E::Copy {})?;

    let permissions = Permissions::from_mode(0o755);
    fs::set_permissions(&temp_path, permissions).with_context(|| E::SetPermissions {
        path: temp_path.to_owned(),
    })?;

    let output = Command::new(temp_path).arg("--version").output()?;
    let new_version = String::from_utf8_lossy(&output.stdout);
    let new_version = new_version
        .trim()
        .trim_start_matches(concat!(env!("CARGO_PKG_NAME"), " "));
    if semver::Version::parse(new_version) > semver::Version::parse(CURRENT_VERSION) {
        info!(
            "Updating up-rs from '{}' to '{}'",
            CURRENT_VERSION, &new_version,
        );
        fs::rename(&temp_path, &up_path).with_context(|| E::Rename {
            from: temp_path.clone(),
            to: up_path.clone(),
        })?;
    } else {
        info!(
            "Skipping up-rs update, current version '{}' and new version '{}'",
            CURRENT_VERSION, &new_version,
        );
    }
    Ok(())
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
