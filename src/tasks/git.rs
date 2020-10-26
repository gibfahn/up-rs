use std::convert::From;

use anyhow::Result;
use displaydoc::Display;
use git2::Remote;
use log::error;
use rayon::prelude::*;
use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;
use thiserror::Error;

use self::GitTaskError as E;
use crate::tasks::ResolveEnv;

pub mod update;

pub const DEFAULT_REMOTE_NAME: &str = "origin";

#[derive(Debug, Default, StructOpt)]
pub struct GitArgs {
    /// URL of git repo to download.
    #[structopt(long)]
    pub git_url: String,
    /// Path to download git repo to.
    #[structopt(long)]
    pub git_path: String,
    /// Remote to set/update.
    #[structopt(long, default_value = DEFAULT_REMOTE_NAME)]
    pub remote: String,
    /// Branch to checkout when cloning/updating. Defaults to default branch for
    /// cloning, and current branch for updating.
    #[structopt(long)]
    pub branch: Option<String>,
}

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
}

// TODO(gib): Pass by reference instead.
#[allow(clippy::clippy::needless_pass_by_value)]
pub(crate) fn run(configs: Vec<GitConfig>) -> Result<()> {
    // TODO(gib): run them in parallel.
    // TODO(gib): continue even if one errors.
    let errors: Vec<_> = configs
        .par_iter()
        .map(|c| update::update(c))
        .filter_map(Result::err)
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        for error in &errors {
            error!("{:?}", error);
        }
        let first_error = errors.into_iter().next().ok_or(E::UnexpectedNone)?;
        Err(first_error)
    }
}

impl From<GitArgs> for GitConfig {
    fn from(item: GitArgs) -> Self {
        Self {
            path: item.git_path,
            remotes: vec![GitRemote {
                name: item.remote,
                push_url: None,
                fetch_url: item.git_url,
            }],
            branch: item.branch,
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

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
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
