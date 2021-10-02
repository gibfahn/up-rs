use std::convert::From;

use clap::Clap;
use color_eyre::eyre::{eyre, Context, Result};
use displaydoc::Display;
use git2::Remote;
use log::error;
use rayon::prelude::*;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use self::GitTaskError as E;
use crate::{opts::GitOptions, tasks::ResolveEnv};

pub mod branch;
pub mod checkout;
pub mod cherry;
pub mod errors;
pub mod fetch;
pub mod merge;
pub mod prune;
pub mod status;
pub mod update;

pub const DEFAULT_REMOTE_NAME: &str = "origin";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GitConfig {
    /// Path to download git repo to.
    pub path: String,
    /// Remote to set/update.
    pub remotes: Vec<GitRemote>,
    /// Branch to checkout when cloning/updating. Defaults to the current branch
    /// when updating, or the default branch of the first remote for
    /// cloning.
    pub branch: Option<String>,
    /// Prune local branches whose changes have already been merged upstream.
    #[serde(default = "prune_default")]
    pub prune: bool,
}

// Serde needs a function to set a default.
const fn prune_default() -> bool {
    false
}

pub(crate) fn run(configs: &[GitConfig]) -> Result<()> {
    let errors: Vec<_> = configs
        .par_iter()
        .map(update::update)
        .filter_map(Result::err)
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        for error in &errors {
            error!("{:?}", error);
        }
        let mut errors_iter = errors.into_iter();
        Err(errors_iter.next().ok_or(E::UnexpectedNone)?)
            .with_context(|| eyre!("{:?}", errors_iter.collect::<Vec<_>>()))
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
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        for config in self.iter_mut() {
            if let Some(branch) = config.branch.as_ref() {
                config.branch = Some(env_fn(branch)?);
            }
            config.path = env_fn(&config.path)?;
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

#[derive(Debug, Default, Clap, Serialize, Deserialize)]
pub struct GitRemote {
    /// Name of the remote to set in git.
    pub name: String,
    /// URL to fetch from. Also used for pushing if `push_url` unset.
    pub fetch_url: String,
    /// URL to push to, defaults to fetch URL.
    pub push_url: Option<String>,
}

impl GitRemote {
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
