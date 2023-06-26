//! The git library task.
use self::GitTaskError as E;
use crate::opts::GitOptions;
use crate::tasks::task::TaskStatus;
use crate::tasks::ResolveEnv;
use crate::tasks::TaskError;
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::Result;
use displaydoc::Display;
use git2::Remote;
use rayon::iter::Either;
use rayon::prelude::*;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::convert::From;
use thiserror::Error;
use tracing::error;

pub mod branch;
pub mod checkout;
pub mod cherry;
pub mod errors;
pub mod fetch;
pub mod merge;
pub mod prune;
pub mod status;
pub mod update;

/// Default git remote name.
pub const DEFAULT_REMOTE_NAME: &str = "origin";

/// `up git` configuration options.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GitConfig {
    /// Path to download git repo to.
    pub path: Utf8PathBuf,
    /// Remote to set/update.
    pub remotes: Vec<GitRemote>,
    /// Branch to checkout when cloning/updating. Defaults to the current branch
    /// when updating, or the default branch of the first remote for
    /// cloning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Prune local branches whose changes have already been merged upstream.
    #[serde(default = "prune_default")]
    pub prune: bool,
}

/// Serde needs a function to set a default, so this sets a default of false.
const fn prune_default() -> bool {
    false
}

/// Run the `up git` task.
pub(crate) fn run(configs: &[GitConfig]) -> Result<TaskStatus> {
    let (statuses, errors): (Vec<_>, Vec<_>) = configs
        .par_iter()
        .map(update::update)
        .partition_map(|x| match x {
            Ok(status) => Either::Left(status),
            Err(e) => Either::Right(e),
        });

    if errors.is_empty() {
        if statuses.iter().all(|s| matches!(s, TaskStatus::Skipped)) {
            Ok(TaskStatus::Skipped)
        } else {
            Ok(TaskStatus::Passed)
        }
    } else {
        for error in &errors {
            error!("{error:?}");
        }
        let first_error = errors.into_iter().next().ok_or(E::UnexpectedNone)?;
        Err(first_error)
    }
}

impl From<GitOptions> for GitConfig {
    fn from(item: GitOptions) -> Self {
        Self {
            path: item.git_path,
            remotes: vec![GitRemote {
                name: item.remote,
                push_url: None,
                fetch_url: item.git_url,
            }],
            branch: item.branch,
            prune: item.prune,
        }
    }
}

impl ResolveEnv for Vec<GitConfig> {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<(), TaskError>
    where
        F: Fn(&str) -> Result<String, TaskError>,
    {
        for config in self.iter_mut() {
            if let Some(branch) = config.branch.as_ref() {
                config.branch = Some(env_fn(branch)?);
            }
            config.path = Utf8PathBuf::from(env_fn(config.path.as_str())?);
            for remote in &mut config.remotes {
                remote.name = env_fn(&remote.name)?;
                remote.push_url = if let Some(push_url) = &remote.push_url {
                    Some(env_fn(push_url)?)
                } else {
                    None
                };
                remote.fetch_url = env_fn(&remote.fetch_url)?;
            }
        }
        Ok(())
    }
}

/// Represents a git remote.
#[derive(Debug, Default, Parser, Serialize, Deserialize)]
pub struct GitRemote {
    /// Name of the remote to set in git.
    pub name: String,
    /// URL to fetch from. Also used for pushing if `push_url` unset.
    pub fetch_url: String,
    /// URL to push to, defaults to fetch URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_url: Option<String>,
}

impl GitRemote {
    /// Create a git remote from a git2-rs remote.
    pub(crate) fn from(remote: &Remote) -> Result<Self> {
        let fetch_url = remote.url().ok_or(E::InvalidRemote)?.to_owned();

        let push_url = match remote.pushurl() {
            Some(url) if url != fetch_url => Some(url.to_owned()),
            _ => None,
        };

        Ok(Self {
            name: remote.name().ok_or(E::InvalidRemote)?.to_owned(),
            fetch_url,
            push_url,
        })
    }
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GitTaskError {
    /// Remote un-named, or invalid UTF-8 name.
    InvalidRemote,
    /// Unexpected None in option.
    UnexpectedNone,
}
