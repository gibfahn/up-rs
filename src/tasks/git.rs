use std::{fs, io, path::PathBuf};

use anyhow::Result;
use displaydoc::Display;
use git2::{ErrorCode, Repository};
use log::debug;
use structopt::StructOpt;
use thiserror::Error;

mod clone;
mod update;

pub const DEFAULT_REMOTE_NAME: &str = "origin";

#[derive(Debug, Default, StructOpt)]
pub struct GitConfig {
    /// URL of git repo to download.
    #[structopt(long)]
    pub git_url: String,
    /// Path to download git repo to.
    #[structopt(long, parse(from_os_str))]
    pub git_path: PathBuf,
    /// Remote to set/update.
    #[structopt(long, default_value = DEFAULT_REMOTE_NAME)]
    pub remote: String,
    /// Branch to checkout when cloning/updating. Defaults to default branch for
    /// cloning, and current branch for updating.
    #[structopt(long)]
    pub branch: Option<String>,
}

pub fn clone_or_update(git_config: GitConfig) -> Result<()> {
    if !git_config.git_path.is_dir() {
        debug!("Dir doesn't exist, creating...");
        fs::create_dir_all(&git_config.git_path).map_err(|e| GitError::CreateDirError {
            path: git_config.git_path.to_path_buf(),
            source: e,
        })?;
    }
    match Repository::open(&git_config.git_path) {
        Ok(repo) => update::update(git_config, &repo),
        Err(e) => {
            if let ErrorCode::NotFound = e.code() {
                clone::clone(git_config)
            } else {
                debug!("Failed to open repository: {:?}\n  {}", e.code(), e);
                Err(e.into())
            }
        }
    }
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GitError {
    /// Failed to create directory '{path}'
    CreateDirError { path: PathBuf, source: io::Error },
}
